// Tests for the CPU and the assembler.
//
//   cargo test                  run everything
//   cargo test decode           run only tests whose name contains "decode"
//   cargo test asm_             run only the assembler tests
//
// Tests whose comment starts with "BUG:" fail right now on purpose.
// Each one points at a real bug. Fix the bug and the test turns green.

use RISC_V_emulator_in_Rust::asm::{self, Operand, lex, lexed2instr, str2reg};
use RISC_V_emulator_in_Rust::cpu::Instruction::{self, *};

// ====================================================================
// Reference encoders
// ====================================================================
// Small functions that build machine code by hand, straight from the
// RISC-V spec, so the tests don't need hex everywhere. They are written
// separately from your assembler on purpose: when two independent
// versions agree, you can trust the answer.

const OP: u32 = 0b0110011; // R-type arithmetic
const OP_IMM: u32 = 0b0010011; // I-type arithmetic
const LOAD: u32 = 0b0000011;
const STORE: u32 = 0b0100011;
const BRANCH: u32 = 0b1100011;
const JAL: u32 = 0b1101111;
const JALR: u32 = 0b1100111;
const LUI: u32 = 0b0110111;
const AUIPC: u32 = 0b0010111;

// add rd, rs1, rs2
fn r(rd: u32, rs1: u32, rs2: u32, f3: u32, f7: u32) -> u32 {
	OP | (rd << 7) | (f3 << 12) | (rs1 << 15) | (rs2 << 20) | (f7 << 25)
}

// addi rd, rs1, imm   /   lw rd, imm(rs1)
fn i(op: u32, rd: u32, rs1: u32, imm: i32, f3: u32) -> u32 {
	op | (rd << 7) | (f3 << 12) | (rs1 << 15) | (((imm as u32) & 0xFFF) << 20)
}

// sw rs2, imm(rs1)
fn s(rs2: u32, rs1: u32, imm: i32, f3: u32) -> u32 {
	let m = imm as u32;
	STORE | ((m & 0x1F) << 7) | (f3 << 12) | (rs1 << 15) | (rs2 << 20) | (((m >> 5) & 0x7F) << 25)
}

// beq rs1, rs2, offset
fn b(rs1: u32, rs2: u32, offset: i32, f3: u32) -> u32 {
	let m = offset as u32;
	BRANCH
		| (((m >> 11) & 1) << 7)
		| (((m >> 1) & 0xF) << 8)
		| (f3 << 12)
		| (rs1 << 15)
		| (rs2 << 20)
		| (((m >> 5) & 0x3F) << 25)
		| (((m >> 12) & 1) << 31)
}

// lui rd, imm20
fn u(op: u32, rd: u32, imm20: u32) -> u32 {
	op | (rd << 7) | ((imm20 & 0xFFFFF) << 12)
}

// jal rd, offset
fn j(rd: u32, offset: i32) -> u32 {
	let m = offset as u32;
	JAL | (rd << 7)
		| (((m >> 12) & 0xFF) << 12)
		| (((m >> 11) & 1) << 20)
		| (((m >> 1) & 0x3FF) << 21)
		| (((m >> 20) & 1) << 31)
}

// Check the encoders themselves against values from the official GNU assembler.
#[test]
fn reference_encoders_match_gnu_as() {
	assert_eq!(r(5, 5, 6, 0, 0), 0x006282B3); // add  x5, x5, x6
	assert_eq!(i(OP_IMM, 5, 0, -1, 0), 0xFFF00293); // addi x5, x0, -1
	assert_eq!(s(5, 8, 0, 2), 0x00542023); // sw   x5, 0(x8)
	assert_eq!(b(6, 7, -8, 1), 0xFE731CE3); // bne  x6, x7, -8
	assert_eq!(j(1, 20), 0x014000EF); // jal  ra, 20
	assert_eq!(u(LUI, 1, 5), 0x000050B7); // lui  ra, 5
}

// ====================================================================
// A tiny CPU wrapper for the tests
// ====================================================================
// Bundles registers, memory and pc together so tests stay short.
// (This is also a preview of the `Cpu` struct you might add to cpu.rs.)

struct Cpu {
	x: [u32; 32],
	mem: Vec<u8>,
	pc: u32,
}

impl Cpu {
	fn new() -> Cpu {
		Cpu { x: [0; 32], mem: vec![0; 1024], pc: 0 }
	}

	// Run one already-decoded instruction. Returns true if the CPU halted.
	fn exec(&mut self, instr: Instruction) -> bool {
		Instruction::execute_instruction(&mut self.x, &mut self.mem, &mut self.pc, instr)
	}

	// Copy machine code into memory starting at address 0.
	fn load(&mut self, program: &[u32]) {
		for (n, word) in program.iter().enumerate() {
			self.mem[n * 4..n * 4 + 4].copy_from_slice(&word.to_le_bytes());
		}
	}

	// fetch -> decode -> execute until ecall. Fails if it never stops.
	fn run(&mut self) {
		for _ in 0..10_000 {
			let mut len = 0;
			let raw = Instruction::fetch_instruction(&self.mem, &mut self.pc, &mut len);
			let instr = Instruction::decode_instruction(raw);
			if self.exec(instr) {
				return;
			}
		}
		panic!("program never reached ecall");
	}
}

// ====================================================================
// CPU: decode (number -> Instruction)
// ====================================================================

#[test]
fn decode_every_instruction() {
	// A list of (machine code, what it should decode to).
	let cases: Vec<(u32, Instruction)> = vec![
		// R-type
		(r(1, 2, 3, 0x0, 0x00), Add { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x0, 0x20), Sub { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x1, 0x00), Sll { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x2, 0x00), Slt { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x3, 0x00), Sltu { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x4, 0x00), Xor { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x5, 0x00), Srl { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x5, 0x20), Sra { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x6, 0x00), Or { rd: 1, rs1: 2, rs2: 3 }),
		(r(1, 2, 3, 0x7, 0x00), And { rd: 1, rs1: 2, rs2: 3 }),
		(r(31, 31, 31, 0x0, 0x00), Add { rd: 31, rs1: 31, rs2: 31 }),
		// I-type arithmetic
		(i(OP_IMM, 4, 5, 100, 0x0), Addi { rd: 4, rs1: 5, imm: 100 }),
		(i(OP_IMM, 4, 5, -1, 0x0), Addi { rd: 4, rs1: 5, imm: -1 }),
		(i(OP_IMM, 4, 5, 2047, 0x0), Addi { rd: 4, rs1: 5, imm: 2047 }),
		(i(OP_IMM, 4, 5, -2048, 0x0), Addi { rd: 4, rs1: 5, imm: -2048 }),
		(i(OP_IMM, 4, 5, -7, 0x2), Slti { rd: 4, rs1: 5, imm: -7 }),
		(i(OP_IMM, 4, 5, 7, 0x3), Sltiu { rd: 4, rs1: 5, imm: 7 }),
		(i(OP_IMM, 4, 5, 0xFF, 0x4), Xori { rd: 4, rs1: 5, imm: 0xFF }),
		(i(OP_IMM, 4, 5, 0xFF, 0x6), Ori { rd: 4, rs1: 5, imm: 0xFF }),
		(i(OP_IMM, 4, 5, 0xFF, 0x7), Andi { rd: 4, rs1: 5, imm: 0xFF }),
		(i(OP_IMM, 4, 5, 3, 0x1), Slli { rd: 4, rs1: 5, shamt: 3 }),
		(i(OP_IMM, 4, 5, 31, 0x5), Srli { rd: 4, rs1: 5, shamt: 31 }),
		(i(OP_IMM, 4, 5, 0x400 | 3, 0x5), Srai { rd: 4, rs1: 5, shamt: 3 }),
		// Loads
		(i(LOAD, 6, 7, 8, 0x0), Lb { rd: 6, rs1: 7, imm: 8 }),
		(i(LOAD, 6, 7, 8, 0x1), Lh { rd: 6, rs1: 7, imm: 8 }),
		(i(LOAD, 6, 7, -4, 0x2), Lw { rd: 6, rs1: 7, imm: -4 }),
		(i(LOAD, 6, 7, 8, 0x4), Lbu { rd: 6, rs1: 7, imm: 8 }),
		(i(LOAD, 6, 7, 8, 0x5), Lhu { rd: 6, rs1: 7, imm: 8 }),
		// Stores
		(s(9, 8, 12, 0x0), Sb { rs1: 8, rs2: 9, imm: 12 }),
		(s(9, 8, 12, 0x1), Sh { rs1: 8, rs2: 9, imm: 12 }),
		(s(9, 8, -4, 0x2), Sw { rs1: 8, rs2: 9, imm: -4 }),
		(s(9, 8, 2047, 0x2), Sw { rs1: 8, rs2: 9, imm: 2047 }),
		(s(9, 8, -2048, 0x2), Sw { rs1: 8, rs2: 9, imm: -2048 }),
		// Branches (offsets are always even)
		(b(1, 2, 16, 0x0), Beq { rs1: 1, rs2: 2, offset: 16 }),
		(b(1, 2, -16, 0x1), Bne { rs1: 1, rs2: 2, offset: -16 }),
		(b(1, 2, 2048, 0x4), Blt { rs1: 1, rs2: 2, offset: 2048 }),
		(b(1, 2, 4094, 0x5), Bge { rs1: 1, rs2: 2, offset: 4094 }),
		(b(1, 2, -4096, 0x6), Bltu { rs1: 1, rs2: 2, offset: -4096 }),
		(b(1, 2, 2, 0x7), Bgeu { rs1: 1, rs2: 2, offset: 2 }),
		// Jumps
		(j(1, 20), Jal { rd: 1, offset: 20 }),
		(j(0, -20), Jal { rd: 0, offset: -20 }),
		(j(1, 1048574), Jal { rd: 1, offset: 1048574 }), // biggest forward jump
		(j(1, -1048576), Jal { rd: 1, offset: -1048576 }), // biggest backward jump
		(i(JALR, 1, 5, -8, 0x0), Jalr { rd: 1, rs1: 5, imm: -8 }),
		// Upper immediates (the decoder keeps the raw 20 bits)
		(u(LUI, 3, 0xABCDE), Lui { rd: 3, imm: 0xABCDE }),
		(u(AUIPC, 3, 0x12345), Auipc { rd: 3, imm: 0x12345 }),
		// System
		(0x00000073, Ecall),
		(0x00100073, Ebreak),
		(0x0FF0000F, Fence), // fence iorw, iorw
	];

	for (raw, expected) in cases {
		// The third argument is a message shown only if the check fails.
		assert_eq!(Instruction::decode_instruction(raw), expected, "while decoding {:#010x}", raw);
	}
}

#[test]
fn decode_invalid_encodings_are_illegal() {
	let bad: Vec<u32> = vec![
		0x00000000,                    // all zeros
		0xFFFFFFFF,                    // all ones
		r(1, 2, 3, 0x0, 0x01),         // ADD with a wrong funct7
		r(1, 2, 3, 0x1, 0x20),         // SLL with a wrong funct7
		i(OP_IMM, 1, 2, 0x200 | 3, 0x1), // SLLI with a wrong funct7
		i(LOAD, 1, 2, 0, 0x3),         // funct3 = 3 is LD, which is RV64 only
		i(LOAD, 1, 2, 0, 0x6),         // funct3 = 6 is not a load
		s(1, 2, 0, 0x3),               // funct3 = 3 is SD, RV64 only
		b(1, 2, 8, 0x2),               // funct3 = 2 is not a branch
		i(JALR, 1, 2, 0, 0x1),         // JALR must have funct3 = 0
		0x00200073,                    // SYSTEM with an unknown immediate
	];
	for raw in bad {
		assert_eq!(Instruction::decode_instruction(raw), Illegal(raw), "while decoding {:#010x}", raw);
	}
}

// ====================================================================
// CPU: fetch (bytes in memory -> number)
// ====================================================================

#[test]
fn fetch_is_little_endian() {
	// add x5, x5, x6 as it sits in memory: lowest byte first
	let mem = vec![0xB3, 0x82, 0x62, 0x00];
	let mut pc = 0;
	let mut len = 0;
	assert_eq!(Instruction::fetch_instruction(&mem, &mut pc, &mut len), 0x006282B3);
	assert_eq!(len, 4);
}

#[test]
fn fetch_from_the_middle_of_memory() {
	let mut mem = vec![0u8; 16];
	mem[8..12].copy_from_slice(&0x00100073u32.to_le_bytes());
	let mut pc = 8;
	let mut len = 0;
	assert_eq!(Instruction::fetch_instruction(&mem, &mut pc, &mut len), 0x00100073);
	assert_eq!(pc, 8); // fetching must not move pc; execute does that
}

#[test]
fn fetch_detects_16_bit_instructions() {
	// If the lowest 2 bits are not 0b11, it's a 2-byte (compressed) instruction.
	let mem = vec![0x01, 0x00, 0x00, 0x00];
	let mut pc = 0;
	let mut len = 0;
	assert_eq!(Instruction::fetch_instruction(&mem, &mut pc, &mut len), 0x0001);
	assert_eq!(len, 2);
}

// `should_panic` means: this test passes only if the code crashes.
// `expected` checks that the crash message contains that text.
#[test]
#[should_panic(expected = "instruction too short")]
fn fetch_past_end_of_memory_panics() {
	let mem = vec![0x13, 0x00]; // first half of a 4-byte instruction, then nothing
	let mut pc = 0;
	let mut len = 0;
	Instruction::fetch_instruction(&mem, &mut pc, &mut len);
}

// ====================================================================
// CPU: execute (Instruction -> changes registers / memory / pc)
// ====================================================================

#[test]
fn execute_add_and_sub() {
	let mut cpu = Cpu::new();
	cpu.x[1] = 10;
	cpu.x[2] = 32;
	cpu.exec(Add { rd: 3, rs1: 1, rs2: 2 });
	assert_eq!(cpu.x[3], 42);
	assert_eq!(cpu.pc, 4); // moved on to the next instruction

	cpu.exec(Sub { rd: 3, rs1: 1, rs2: 2 }); // 10 - 32
	assert_eq!(cpu.x[3], -22i32 as u32);

	// overflow wraps around instead of crashing
	cpu.x[1] = 0xFFFF_FFFF;
	cpu.x[2] = 1;
	cpu.exec(Add { rd: 3, rs1: 1, rs2: 2 });
	assert_eq!(cpu.x[3], 0);
	cpu.exec(Sub { rd: 3, rs1: 0, rs2: 2 }); // 0 - 1
	assert_eq!(cpu.x[3], 0xFFFF_FFFF);
}

#[test]
fn execute_same_register_everywhere() {
	// add x1, x1, x1 must read x1 before writing it
	let mut cpu = Cpu::new();
	cpu.x[1] = 21;
	cpu.exec(Add { rd: 1, rs1: 1, rs2: 1 });
	assert_eq!(cpu.x[1], 42);
}

#[test]
fn execute_logic() {
	let mut cpu = Cpu::new();
	cpu.x[1] = 0b1100;
	cpu.x[2] = 0b1010;
	cpu.exec(Xor { rd: 3, rs1: 1, rs2: 2 });
	assert_eq!(cpu.x[3], 0b0110);
	cpu.exec(Or { rd: 3, rs1: 1, rs2: 2 });
	assert_eq!(cpu.x[3], 0b1110);
	cpu.exec(And { rd: 3, rs1: 1, rs2: 2 });
	assert_eq!(cpu.x[3], 0b1000);
}

#[test]
fn execute_register_shifts() {
	let mut cpu = Cpu::new();
	cpu.x[1] = 1;
	cpu.x[2] = 4;
	cpu.exec(Sll { rd: 3, rs1: 1, rs2: 2 });
	assert_eq!(cpu.x[3], 16);

	// only the low 5 bits of the shift amount count: 33 behaves like 1
	cpu.x[2] = 33;
	cpu.exec(Sll { rd: 3, rs1: 1, rs2: 2 });
	assert_eq!(cpu.x[3], 2);

	cpu.x[1] = 0x8000_0000;
	cpu.x[2] = 4;
	cpu.exec(Srl { rd: 3, rs1: 1, rs2: 2 }); // logical: fills with 0s
	assert_eq!(cpu.x[3], 0x0800_0000);
	cpu.exec(Sra { rd: 3, rs1: 1, rs2: 2 }); // arithmetic: copies the sign bit
	assert_eq!(cpu.x[3], 0xF800_0000);
}

#[test]
fn execute_slt_signed_vs_unsigned() {
	let mut cpu = Cpu::new();
	cpu.x[1] = -1i32 as u32; // 0xFFFFFFFF
	cpu.x[2] = 1;
	cpu.exec(Slt { rd: 3, rs1: 1, rs2: 2 }); // signed: -1 < 1
	assert_eq!(cpu.x[3], 1);
	cpu.exec(Sltu { rd: 3, rs1: 1, rs2: 2 }); // unsigned: 0xFFFFFFFF < 1 is false
	assert_eq!(cpu.x[3], 0);
	cpu.exec(Slt { rd: 3, rs1: 2, rs2: 2 }); // equal is not "less than"
	assert_eq!(cpu.x[3], 0);
}

#[test]
fn execute_immediate_arithmetic() {
	let mut cpu = Cpu::new();
	cpu.x[1] = 10;
	cpu.exec(Addi { rd: 2, rs1: 1, imm: -3 });
	assert_eq!(cpu.x[2], 7);

	cpu.x[1] = -5i32 as u32;
	cpu.exec(Slti { rd: 2, rs1: 1, imm: 0 }); // -5 < 0
	assert_eq!(cpu.x[2], 1);

	// sltiu: the immediate is sign-extended, THEN compared unsigned.
	// So imm = -1 means 0xFFFFFFFF, and almost everything is below it.
	cpu.x[1] = 5;
	cpu.exec(Sltiu { rd: 2, rs1: 1, imm: -1 });
	assert_eq!(cpu.x[2], 1);

	cpu.x[1] = 0x0000_0F0F;
	cpu.exec(Xori { rd: 2, rs1: 1, imm: -1 }); // xori with -1 flips every bit
	assert_eq!(cpu.x[2], 0xFFFF_F0F0);
	cpu.exec(Ori { rd: 2, rs1: 1, imm: 0xF0 });
	assert_eq!(cpu.x[2], 0x0000_0FFF);
	cpu.exec(Andi { rd: 2, rs1: 1, imm: 0xFF });
	assert_eq!(cpu.x[2], 0x0000_000F);
	cpu.exec(Andi { rd: 2, rs1: 1, imm: -1 }); // and with all ones keeps everything
	assert_eq!(cpu.x[2], 0x0000_0F0F);
}

#[test]
fn execute_immediate_shifts() {
	let mut cpu = Cpu::new();
	cpu.x[1] = 0x8000_0001;
	cpu.exec(Slli { rd: 2, rs1: 1, shamt: 1 });
	assert_eq!(cpu.x[2], 0x0000_0002); // the top bit falls off
	cpu.exec(Srli { rd: 2, rs1: 1, shamt: 31 });
	assert_eq!(cpu.x[2], 1);
	cpu.exec(Srai { rd: 2, rs1: 1, shamt: 31 });
	assert_eq!(cpu.x[2], 0xFFFF_FFFF);
}

#[test]
fn execute_loads() {
	let mut cpu = Cpu::new();
	cpu.mem[100..104].copy_from_slice(&[0x80, 0xFF, 0x34, 0x12]);
	cpu.x[1] = 100;

	cpu.exec(Lb { rd: 2, rs1: 1, imm: 0 }); // 0x80 sign-extended
	assert_eq!(cpu.x[2], 0xFFFF_FF80);
	cpu.exec(Lbu { rd: 2, rs1: 1, imm: 0 }); // 0x80 zero-extended
	assert_eq!(cpu.x[2], 0x0000_0080);
	cpu.exec(Lh { rd: 2, rs1: 1, imm: 0 }); // 0xFF80 sign-extended
	assert_eq!(cpu.x[2], 0xFFFF_FF80);
	cpu.exec(Lhu { rd: 2, rs1: 1, imm: 0 });
	assert_eq!(cpu.x[2], 0x0000_FF80);
	cpu.exec(Lh { rd: 2, rs1: 1, imm: 2 }); // 0x1234 is positive, nothing to extend
	assert_eq!(cpu.x[2], 0x0000_1234);
	cpu.exec(Lw { rd: 2, rs1: 1, imm: 0 });
	assert_eq!(cpu.x[2], 0x1234_FF80);

	// negative offset: address = 104 + (-4) = 100
	cpu.x[1] = 104;
	cpu.exec(Lw { rd: 2, rs1: 1, imm: -4 });
	assert_eq!(cpu.x[2], 0x1234_FF80);
}

#[test]
fn execute_stores() {
	let mut cpu = Cpu::new();
	cpu.x[1] = 200; // base address
	cpu.x[2] = 0xAABB_CCDD;

	cpu.exec(Sw { rs1: 1, rs2: 2, imm: 0 });
	assert_eq!(&cpu.mem[200..204], &[0xDD, 0xCC, 0xBB, 0xAA]);

	// sb writes 1 byte and leaves its neighbours alone
	cpu.exec(Sb { rs1: 1, rs2: 0, imm: 1 });
	assert_eq!(&cpu.mem[200..204], &[0xDD, 0x00, 0xBB, 0xAA]);

	// sh writes 2 bytes
	cpu.exec(Sh { rs1: 1, rs2: 0, imm: 2 });
	assert_eq!(&cpu.mem[200..204], &[0xDD, 0x00, 0x00, 0x00]);

	// negative offset
	cpu.exec(Sw { rs1: 1, rs2: 2, imm: -4 });
	assert_eq!(&cpu.mem[196..200], &[0xDD, 0xCC, 0xBB, 0xAA]);
}

#[test]
fn execute_store_then_load_roundtrip() {
	let mut cpu = Cpu::new();
	cpu.x[5] = 0xDEAD_BEEF;
	cpu.x[8] = 256;
	cpu.exec(Sw { rs1: 8, rs2: 5, imm: 0 });
	cpu.exec(Lw { rd: 9, rs1: 8, imm: 0 });
	assert_eq!(cpu.x[9], 0xDEAD_BEEF);
}

// Puts `a` in x1 and `b` in x2, runs the branch at pc = 100 with offset +8,
// and says whether it jumped.
fn branch_taken(make: fn(usize, usize, i32) -> Instruction, a: i32, b: i32) -> bool {
	let mut cpu = Cpu::new();
	cpu.pc = 100;
	cpu.x[1] = a as u32;
	cpu.x[2] = b as u32;
	cpu.exec(make(1, 2, 8));
	match cpu.pc {
		108 => true,
		104 => false,
		other => panic!("branch went to a strange pc: {}", other),
	}
}

#[test]
fn execute_branches() {
	// These little closures turn "Beq" into a function we can pass around.
	let beq = |rs1, rs2, offset| Beq { rs1, rs2, offset };
	let bne = |rs1, rs2, offset| Bne { rs1, rs2, offset };
	let blt = |rs1, rs2, offset| Blt { rs1, rs2, offset };
	let bge = |rs1, rs2, offset| Bge { rs1, rs2, offset };
	let bltu = |rs1, rs2, offset| Bltu { rs1, rs2, offset };
	let bgeu = |rs1, rs2, offset| Bgeu { rs1, rs2, offset };

	assert!(branch_taken(beq, 5, 5));
	assert!(!branch_taken(beq, 5, 6));

	assert!(branch_taken(bne, 5, 6));
	assert!(!branch_taken(bne, 5, 5));

	assert!(branch_taken(blt, -1, 1)); // signed
	assert!(!branch_taken(blt, 1, -1));
	assert!(!branch_taken(blt, 3, 3));

	assert!(branch_taken(bge, 1, -1));
	assert!(branch_taken(bge, 3, 3)); // >= includes equal
	assert!(!branch_taken(bge, -1, 1));

	assert!(branch_taken(bltu, 1, -1)); // unsigned: 1 < 0xFFFFFFFF
	assert!(!branch_taken(bltu, -1, 1));

	assert!(branch_taken(bgeu, -1, 1));
	assert!(branch_taken(bgeu, 3, 3));
	assert!(!branch_taken(bgeu, 1, -1));
}

#[test]
fn execute_branch_backwards() {
	let mut cpu = Cpu::new();
	cpu.pc = 100;
	cpu.exec(Beq { rs1: 0, rs2: 0, offset: -8 });
	assert_eq!(cpu.pc, 92);
}

#[test]
fn execute_jal() {
	let mut cpu = Cpu::new();
	cpu.pc = 40;
	cpu.exec(Jal { rd: 1, offset: 20 });
	assert_eq!(cpu.x[1], 44); // ra = address of the next instruction
	assert_eq!(cpu.pc, 60);

	cpu.exec(Jal { rd: 0, offset: -60 }); // "j": jump without saving
	assert_eq!(cpu.x[0], 0);
	assert_eq!(cpu.pc, 0);
}

#[test]
fn execute_jalr() {
	let mut cpu = Cpu::new();
	cpu.pc = 40;
	cpu.x[5] = 200;
	cpu.exec(Jalr { rd: 1, rs1: 5, imm: -8 });
	assert_eq!(cpu.pc, 192);
	assert_eq!(cpu.x[1], 44);

	// the lowest bit of the target is always cleared
	cpu.x[5] = 201;
	cpu.exec(Jalr { rd: 1, rs1: 5, imm: 0 });
	assert_eq!(cpu.pc, 200);

	// rd == rs1: the target must be computed before rd is overwritten
	cpu.pc = 40;
	cpu.x[1] = 300;
	cpu.exec(Jalr { rd: 1, rs1: 1, imm: 0 });
	assert_eq!(cpu.pc, 300);
	assert_eq!(cpu.x[1], 44);
}

#[test]
fn execute_lui_and_auipc() {
	let mut cpu = Cpu::new();
	cpu.exec(Lui { rd: 1, imm: 5 });
	assert_eq!(cpu.x[1], 5 << 12);
	cpu.exec(Lui { rd: 1, imm: 0xFFFFF });
	assert_eq!(cpu.x[1], 0xFFFF_F000);

	cpu.pc = 0x100;
	cpu.exec(Auipc { rd: 2, imm: 1 });
	assert_eq!(cpu.x[2], 0x1100); // pc + (1 << 12)
	assert_eq!(cpu.pc, 0x104);
}

#[test]
fn execute_fence_does_nothing() {
	let mut cpu = Cpu::new();
	cpu.x[1] = 7;
	let halted = cpu.exec(Fence);
	assert!(!halted);
	assert_eq!(cpu.x[1], 7);
	assert_eq!(cpu.pc, 4);
}

#[test]
fn execute_ecall_halts_and_others_dont() {
	let mut cpu = Cpu::new();
	assert!(!cpu.exec(Addi { rd: 1, rs1: 0, imm: 1 }));
	assert!(cpu.exec(Ecall));
}

#[test]
fn x0_is_always_zero() {
	let mut cpu = Cpu::new();
	cpu.mem[0..4].copy_from_slice(&[1, 2, 3, 4]);
	cpu.exec(Addi { rd: 0, rs1: 0, imm: 123 });
	assert_eq!(cpu.x[0], 0);
	cpu.exec(Lui { rd: 0, imm: 1 });
	assert_eq!(cpu.x[0], 0);
	cpu.exec(Lw { rd: 0, rs1: 0, imm: 0 });
	assert_eq!(cpu.x[0], 0);
	cpu.exec(Jal { rd: 0, offset: 8 });
	assert_eq!(cpu.x[0], 0);
}

#[test]
#[should_panic(expected = "illegal instruction")]
fn execute_illegal_instruction_panics() {
	let mut cpu = Cpu::new();
	cpu.exec(Illegal(0));
}

// ====================================================================
// CPU: whole programs
// ====================================================================

#[test]
fn program_gauss_sum() {
	// 1 + 2 + ... + 10, the same program as test/1.asm
	let mut cpu = Cpu::new();
	cpu.load(&[
		i(OP_IMM, 5, 0, 0, 0),  // 0:  addi x5, x0, 0
		i(OP_IMM, 6, 0, 1, 0),  // 4:  addi x6, x0, 1
		i(OP_IMM, 7, 0, 11, 0), // 8:  addi x7, x0, 11
		r(5, 5, 6, 0, 0),       // 12: add  x5, x5, x6   <- loop
		i(OP_IMM, 6, 6, 1, 0),  // 16: addi x6, x6, 1
		b(6, 7, -8, 1),         // 20: bne  x6, x7, loop
		0x00000073,             // 24: ecall
	]);
	cpu.run();
	assert_eq!(cpu.x[5], 55);
}

#[test]
fn program_function_call() {
	// main calls double(21) and gets 42 back
	let mut cpu = Cpu::new();
	cpu.load(&[
		i(OP_IMM, 10, 0, 21, 0), // 0:  addi a0, x0, 21
		j(1, 12),                // 4:  jal  ra, double  (16)
		i(OP_IMM, 11, 10, 0, 0), // 8:  addi a1, a0, 0   (copy the result)
		0x00000073,              // 12: ecall
		r(10, 10, 10, 0, 0),     // 16: double: add a0, a0, a0
		i(JALR, 0, 1, 0, 0),     // 20: jalr x0, ra, 0   (return)
	]);
	cpu.run();
	assert_eq!(cpu.x[10], 42);
	assert_eq!(cpu.x[11], 42);
	assert_eq!(cpu.x[1], 8); // return address
}

#[test]
fn program_copy_bytes() {
	// copies 4 bytes from address 512 to address 768, one byte at a time
	let mut cpu = Cpu::new();
	cpu.mem[512..516].copy_from_slice(&[10, 20, 30, 40]);
	cpu.load(&[
		i(OP_IMM, 1, 0, 512, 0), // 0:  addi x1, x0, 512   (source)
		i(OP_IMM, 2, 0, 768, 0), // 4:  addi x2, x0, 768   (destination)
		i(OP_IMM, 3, 0, 4, 0),   // 8:  addi x3, x0, 4     (count)
		i(LOAD, 4, 1, 0, 4),     // 12: lbu  x4, 0(x1)     <- loop
		s(4, 2, 0, 0),           // 16: sb   x4, 0(x2)
		i(OP_IMM, 1, 1, 1, 0),   // 20: addi x1, x1, 1
		i(OP_IMM, 2, 2, 1, 0),   // 24: addi x2, x2, 1
		i(OP_IMM, 3, 3, -1, 0),  // 28: addi x3, x3, -1
		b(3, 0, -20, 1),         // 32: bne  x3, x0, loop
		0x00000073,              // 36: ecall
	]);
	cpu.run();
	assert_eq!(&cpu.mem[768..772], &[10, 20, 30, 40]);
}

// ====================================================================
// Assembler: lex (text -> list of words)
// ====================================================================

#[test]
fn lex_splits_words_and_commas() {
	assert_eq!(lex("add x5, x5, x6"), vec!["add", "x5", ",", "x5", ",", "x6"]);
}

#[test]
fn lex_ignores_extra_spaces_and_tabs() {
	assert_eq!(lex("  \tadd   x5 ,x5,\tx6  "), vec!["add", "x5", ",", "x5", ",", "x6"]);
}

#[test]
fn lex_ignores_comments() {
	assert_eq!(lex("addi x5, x0, 1 ; comment"), vec!["addi", "x5", ",", "x0", ",", "1"]);
	assert_eq!(lex("addi x5, x0, 1 # comment"), vec!["addi", "x5", ",", "x0", ",", "1"]);
	assert!(lex("; only a comment").is_empty());
}

#[test]
fn lex_empty_line() {
	assert!(lex("").is_empty());
	assert!(lex("    ").is_empty());
}

#[test]
fn lex_handles_parentheses() {
	assert_eq!(lex("sw x5, -4(x8)"), vec!["sw", "x5", ",", "-4", "(", "x8", ")"]);
}

#[test]
fn lex_handles_labels() {
	assert_eq!(lex("loop: add x1, x1, x2"), vec!["loop", ":", "add", "x1", ",", "x1", ",", "x2"]);
	assert_eq!(lex("end:"), vec!["end", ":"]);
}

#[test]
fn lex_keeps_strings_together() {
	// spaces, commas and ';' inside quotes are part of the string
	assert_eq!(lex(r#".ascii "a, b; c""#), vec![".ascii", "\"a, b; c\""]);
}

// ====================================================================
// Assembler: parse (list of words -> AsmInstr)
// ====================================================================

fn parse(line: &str) -> Option<asm::AsmInstr> {
	lexed2instr(&lex(line))
}

#[test]
fn parse_simple_instruction() {
	let instr = parse("add x5, x5, x6").unwrap();
	assert_eq!(instr.label, None);
	assert_eq!(instr.name.as_deref(), Some("add"));
	assert_eq!(instr.args.len(), 3);
	assert!(matches!(&instr.args[0], Operand::Direct(s) if s == "x5"));
}

#[test]
fn parse_no_arguments() {
	let instr = parse("ecall").unwrap();
	assert_eq!(instr.name.as_deref(), Some("ecall"));
	assert!(instr.args.is_empty());
}

#[test]
fn parse_label_only() {
	let instr = parse("loop:").unwrap();
	assert_eq!(instr.label.as_deref(), Some("loop"));
	assert_eq!(instr.name, None);
}

#[test]
fn parse_label_and_instruction() {
	let instr = parse("loop: add x5, x5, x6").unwrap();
	assert_eq!(instr.label.as_deref(), Some("loop"));
	assert_eq!(instr.name.as_deref(), Some("add"));
	assert_eq!(instr.args.len(), 3);
}

#[test]
fn parse_empty_line() {
	let instr = parse("").unwrap();
	assert_eq!(instr.label, None);
	assert_eq!(instr.name, None);
}

#[test]
fn parse_offset_and_base() {
	let instr = parse("lw x9, -4(x8)").unwrap();
	match &instr.args[1] {
		Operand::Indirect { offset, base } => {
			assert_eq!(offset, "-4");
			assert_eq!(base, "x8");
		}
		other => panic!("expected Indirect, got {:?}", other),
	}
}

#[test]
fn parse_base_without_offset() {
	let instr = parse("lw x9, (x8)").unwrap();
	match &instr.args[1] {
		Operand::Indirect { offset, base } => {
			assert_eq!(offset, "");
			assert_eq!(base, "x8");
		}
		other => panic!("expected Indirect, got {:?}", other),
	}
}

#[test]
fn parse_bad_syntax_returns_none() {
	assert!(parse("add x1 x2").is_none()); // missing comma
	assert!(parse("lw x1, 4(x2").is_none()); // missing ')'
}

// ====================================================================
// Assembler: register names
// ====================================================================

fn reg(name: &str) -> Option<u32> {
	str2reg(&name.to_string())
}

#[test]
fn register_names_numeric() {
	for n in 0..32 {
		assert_eq!(reg(&format!("x{}", n)), Some(n), "x{}", n);
	}
}

#[test]
fn register_names_abi() {
	// The official names, in order from x0 to x31.
	let names = [
		"zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", //
		"s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5", //
		"a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", //
		"s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
	];
	for (n, name) in names.iter().enumerate() {
		assert_eq!(reg(name), Some(n as u32), "{}", name);
	}
	assert_eq!(reg("fp"), Some(8)); // fp is another name for s0
}

#[test]
fn register_names_invalid() {
	for name in ["x32", "x100", "t7", "a8", "s12", "", "y1", "hello", "xa"] {
		assert_eq!(reg(name), None, "{:?} should not be a register", name);
	}
}

// BUG: str2reg does `s[1] - b'0'` on bytes. For a character that comes
// before '0' (like '-'), that subtraction goes below zero and crashes.
#[test]
fn register_names_weird_input_does_not_crash() {
	assert_eq!(reg("x-1"), None);
}

// ====================================================================
// Assembler: bit helpers I, E, B
// ====================================================================

#[test]
fn bit_helper_i_takes_a_range() {
	assert_eq!(asm::I(0xABCD)(7, 0), 0xCD);
	assert_eq!(asm::I(0xABCD)(15, 8), 0xAB);
	assert_eq!(asm::I(0xABCD)(0, 7), 0xCD); // order doesn't matter
}

#[test]
fn bit_helper_b_takes_one_bit() {
	assert_eq!(asm::B(0b100)(2), 1);
	assert_eq!(asm::B(0b100)(1), 0);
	assert_eq!(asm::B(0x8000_0000)(31), 1);
}

#[test]
fn bit_helper_e_sign_extends() {
	assert_eq!(asm::E(0x800)(11), 0xFFFF_F800); // bit 11 set -> fill with 1s
	assert_eq!(asm::E(0x7FF)(11), 0x0000_07FF); // bit 11 clear -> unchanged
}

// BUG: for the full range 31..0, `1 << (31 - 0 + 1)` is `1 << 32`, which
// is too big for a u32 and crashes in debug builds.
#[test]
fn bit_helper_i_full_range() {
	assert_eq!(asm::I(0xFFFF_FFFF)(31, 0), 0xFFFF_FFFF);
}

// ====================================================================
// Assembler: full output
// ====================================================================
// The assembler prints its result instead of returning it, so these tests
// run the real `asm_bin` program, feed it text, and read what it prints.
// (Once `assemble` returns a Vec<u32>, you can call it directly instead.)

fn assemble(source: &str) -> Vec<u32> {
	use std::io::Write;
	use std::process::{Command, Stdio};

	let mut child = Command::new(env!("CARGO_BIN_EXE_asm_bin"))
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("could not start asm_bin");
	child.stdin.take().unwrap().write_all(source.as_bytes()).unwrap();
	let out = child.wait_with_output().unwrap();

	assert!(
		out.status.success(),
		"the assembler crashed on:\n{}\n\nerror:\n{}",
		source,
		String::from_utf8_lossy(&out.stderr)
	);
	out.stdout.chunks(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())).collect()
}

// Assembles one line and checks it produced exactly one instruction.
fn assemble_one(line: &str) -> u32 {
	let words = assemble(line);
	assert_eq!(words.len(), 1, "expected one instruction from {:?}", line);
	words[0]
}

#[test]
fn asm_r_type() {
	assert_eq!(assemble_one("add x1, x2, x3"), r(1, 2, 3, 0x0, 0x00));
	assert_eq!(assemble_one("sub x1, x2, x3"), r(1, 2, 3, 0x0, 0x20));
	assert_eq!(assemble_one("sll x1, x2, x3"), r(1, 2, 3, 0x1, 0x00));
	assert_eq!(assemble_one("xor x1, x2, x3"), r(1, 2, 3, 0x4, 0x00));
	assert_eq!(assemble_one("srl x1, x2, x3"), r(1, 2, 3, 0x5, 0x00));
	assert_eq!(assemble_one("sra x1, x2, x3"), r(1, 2, 3, 0x5, 0x20));
}

// BUG: asm.rs uses funct3 0x6 for slt. The spec says 0x2.
#[test]
fn asm_slt() {
	assert_eq!(assemble_one("slt x1, x2, x3"), r(1, 2, 3, 0x2, 0x00));
}

// BUG: asm.rs uses funct3 0x7 for sltu. The spec says 0x3.
#[test]
fn asm_sltu() {
	assert_eq!(assemble_one("sltu x1, x2, x3"), r(1, 2, 3, 0x3, 0x00));
}

// BUG: asm.rs uses funct3 0x2 for or. The spec says 0x6.
#[test]
fn asm_or() {
	assert_eq!(assemble_one("or x1, x2, x3"), r(1, 2, 3, 0x6, 0x00));
}

// BUG: asm.rs uses funct3 0x3 for and. The spec says 0x7.
#[test]
fn asm_and() {
	assert_eq!(assemble_one("and x1, x2, x3"), r(1, 2, 3, 0x7, 0x00));
}

#[test]
fn asm_i_type() {
	assert_eq!(assemble_one("addi x1, x2, 100"), i(OP_IMM, 1, 2, 100, 0x0));
	assert_eq!(assemble_one("addi x1, x2, -1"), i(OP_IMM, 1, 2, -1, 0x0));
	assert_eq!(assemble_one("addi x1, x2, 2047"), i(OP_IMM, 1, 2, 2047, 0x0));
	assert_eq!(assemble_one("addi x1, x2, -2048"), i(OP_IMM, 1, 2, -2048, 0x0));
	assert_eq!(assemble_one("slti x1, x2, 5"), i(OP_IMM, 1, 2, 5, 0x2));
	assert_eq!(assemble_one("sltiu x1, x2, 5"), i(OP_IMM, 1, 2, 5, 0x3));
	assert_eq!(assemble_one("xori x1, x2, 5"), i(OP_IMM, 1, 2, 5, 0x4));
	assert_eq!(assemble_one("ori x1, x2, 5"), i(OP_IMM, 1, 2, 5, 0x6));
	assert_eq!(assemble_one("andi x1, x2, 5"), i(OP_IMM, 1, 2, 5, 0x7));
}

#[test]
fn asm_shift_immediates() {
	assert_eq!(assemble_one("slli x1, x2, 3"), i(OP_IMM, 1, 2, 3, 0x1));
	assert_eq!(assemble_one("srli x1, x2, 3"), i(OP_IMM, 1, 2, 3, 0x5));
	assert_eq!(assemble_one("srai x1, x2, 3"), i(OP_IMM, 1, 2, 0x400 | 3, 0x5));
}

#[test]
fn asm_jalr() {
	assert_eq!(assemble_one("jalr x1, x5, -8"), i(JALR, 1, 5, -8, 0x0));
}

#[test]
fn asm_loads() {
	assert_eq!(assemble_one("lb x1, 8(x2)"), i(LOAD, 1, 2, 8, 0x0));
	assert_eq!(assemble_one("lh x1, 8(x2)"), i(LOAD, 1, 2, 8, 0x1));
	assert_eq!(assemble_one("lw x1, 8(x2)"), i(LOAD, 1, 2, 8, 0x2));
	assert_eq!(assemble_one("lbu x1, 8(x2)"), i(LOAD, 1, 2, 8, 0x4));
	assert_eq!(assemble_one("lhu x1, 8(x2)"), i(LOAD, 1, 2, 8, 0x5));
	assert_eq!(assemble_one("lw x1, -4(x2)"), i(LOAD, 1, 2, -4, 0x2));
	assert_eq!(assemble_one("lw x1, (x2)"), i(LOAD, 1, 2, 0, 0x2));
}

#[test]
fn asm_stores() {
	assert_eq!(assemble_one("sb x5, 12(x8)"), s(5, 8, 12, 0x0));
	assert_eq!(assemble_one("sh x5, 12(x8)"), s(5, 8, 12, 0x1));
	assert_eq!(assemble_one("sw x5, 12(x8)"), s(5, 8, 12, 0x2));
	assert_eq!(assemble_one("sw x5, -4(x8)"), s(5, 8, -4, 0x2));
	assert_eq!(assemble_one("sw x5, 2047(x8)"), s(5, 8, 2047, 0x2));
	assert_eq!(assemble_one("sw x5, (x8)"), s(5, 8, 0, 0x2));
}

#[test]
fn asm_system() {
	assert_eq!(assemble_one("ecall"), 0x00000073);
	assert_eq!(assemble_one("ebreak"), 0x00100073);
}

#[test]
fn asm_abi_register_names() {
	assert_eq!(assemble_one("add a0, sp, ra"), r(10, 2, 1, 0x0, 0x00));
	assert_eq!(assemble_one("addi t0, zero, 1"), i(OP_IMM, 5, 0, 1, 0x0));
}

#[test]
fn asm_branch_forward() {
	let words = assemble("beq x1, x2, skip\naddi x0, x0, 0\nskip:\necall");
	assert_eq!(words.len(), 3);
	assert_eq!(words[0], b(1, 2, 8, 0x0)); // skip is 8 bytes ahead
}

#[test]
fn asm_branch_backward() {
	let words = assemble("loop:\naddi x1, x1, 1\nbne x1, x2, loop");
	assert_eq!(words[1], b(1, 2, -4, 0x1)); // loop is 4 bytes back
}

#[test]
fn asm_every_branch() {
	let branches = [("beq", 0x0), ("bne", 0x1), ("blt", 0x4), ("bge", 0x5), ("bltu", 0x6), ("bgeu", 0x7)];
	for (name, f3) in branches {
		let words = assemble(&format!("top:\naddi x0, x0, 0\n{} x1, x2, top", name));
		assert_eq!(words[1], b(1, 2, -4, f3), "{}", name);
	}
}

#[test]
fn asm_labels_and_comments_take_no_space() {
	let words = assemble("start:\n\n   ; just a comment\n# another one\nadd x1, x2, x3\nend:");
	assert_eq!(words, vec![r(1, 2, 3, 0x0, 0x00)]);
}

#[test]
fn asm_label_on_same_line_as_instruction() {
	let words = assemble("loop: addi x1, x1, 1\nbne x1, x2, loop");
	assert_eq!(words, vec![i(OP_IMM, 1, 1, 1, 0x0), b(1, 2, -4, 0x1)]);
}

// BUG: emit_u takes bits 31..12 of the number you wrote, so `lui x1, 5`
// becomes 0. It should put your number into bits 31..12 instead.
#[test]
fn asm_lui() {
	assert_eq!(assemble_one("lui x1, 5"), u(LUI, 1, 5));
}

// BUG: same problem as asm_lui.
#[test]
fn asm_auipc() {
	assert_eq!(assemble_one("auipc x1, 5"), u(AUIPC, 1, 5));
}

// BUG: emit_j checks for 2 arguments, then reads instr.args[2] (the 3rd one),
// so every jal crashes the assembler.
#[test]
fn asm_jal() {
	let words = assemble("jal ra, end\naddi x0, x0, 0\nend:\necall");
	assert_eq!(words[0], j(1, 8));
}

// ====================================================================
// Assembler + CPU together
// ====================================================================

#[test]
fn asm_then_run_gauss_sum() {
	// test/1.asm without the ebreak (the debugger would wait for typing)
	let source = "
		addi x5, x0, 0     ; sum = 0
		addi x6, x0, 1     ; i = 1
		addi x7, x0, 11    ; limit = 11
	label:
		add  x5, x5, x6    ; sum += i
		addi x6, x6, 1     ; i += 1
		bne  x6, x7, label
		addi x8, x0, 256   ; x8 = address 256
		sw   x5, 0(x8)     ; mem[256] = sum
		lw   x9, 0(x8)     ; x9 = mem[256]
		ecall
	";
	let mut cpu = Cpu::new();
	cpu.load(&assemble(source));
	cpu.run();
	assert_eq!(cpu.x[5], 55);
	assert_eq!(cpu.x[9], 55);
	assert_eq!(&cpu.mem[256..260], &[55, 0, 0, 0]);
}
