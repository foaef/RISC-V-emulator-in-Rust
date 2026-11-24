use crate::asm;
use std::io;

#[derive(Debug)]
pub struct Cpu {
    pub x: [u64; 32],
	pub pc: u64,
    pub mem: Vec<u8>,
}

impl Cpu {
    pub fn new(mem: Vec<u8>) -> Cpu {
        Cpu{ x: [0; 32], pc: 0, mem: mem }
    }

    pub fn run(&mut self) {
        loop {
            let raw: u32 = Instruction::fetch_instruction(&self.mem, self.pc);
            let instr: Instruction = Instruction::decode_instruction(raw);
            println!("pc={:#06x}  raw={:#010x}  {:?}", &self.pc, raw, instr);

            if Instruction::execute_instruction(&mut self.x, &mut self.mem, &mut self.pc, instr){
                break;
            }
	    }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
// ---- R-type: rd = rs1 <op> rs2 ----
	Add  { rd: usize, rs1: usize, rs2: usize },
	Sub  { rd: usize, rs1: usize, rs2: usize },
	Sll  { rd: usize, rs1: usize, rs2: usize },
	Slt  { rd: usize, rs1: usize, rs2: usize },
	Sltu { rd: usize, rs1: usize, rs2: usize },
	Xor  { rd: usize, rs1: usize, rs2: usize },
	Srl  { rd: usize, rs1: usize, rs2: usize },
	Sra  { rd: usize, rs1: usize, rs2: usize },
	Or   { rd: usize, rs1: usize, rs2: usize },
	And  { rd: usize, rs1: usize, rs2: usize },

	// ---- I-type arithmetic: rd = rs1 <op> imm ----
	Addi  { rd: usize, rs1: usize, imm: i64 },
	Slti  { rd: usize, rs1: usize, imm: i64 },
	Sltiu { rd: usize, rs1: usize, imm: i64 },
	Xori  { rd: usize, rs1: usize, imm: i64 },
	Ori   { rd: usize, rs1: usize, imm: i64 },
	Andi  { rd: usize, rs1: usize, imm: i64 },
	Slli  { rd: usize, rs1: usize, shamt: u32 },
	Srli  { rd: usize, rs1: usize, shamt: u32 },
	Srai  { rd: usize, rs1: usize, shamt: u32 },

	// ---- Loads: rd = mem[rs1 + imm] ----
	Lb  { rd: usize, rs1: usize, imm: i64 },
	Lh  { rd: usize, rs1: usize, imm: i64 },
	Lw  { rd: usize, rs1: usize, imm: i64 },
	Ld  { rd: usize, rs1: usize, imm: i64 }, // RV64
	Lbu { rd: usize, rs1: usize, imm: i64 },
	Lhu { rd: usize, rs1: usize, imm: i64 },
	Lwu { rd: usize, rs1: usize, imm: i64 }, // RV64

	// ---- Stores: mem[rs1 + imm] = rs2 ----
	Sb { rs1: usize, rs2: usize, imm: i64 },
	Sh { rs1: usize, rs2: usize, imm: i64 },
	Sw { rs1: usize, rs2: usize, imm: i64 },
	Sd { rs1: usize, rs2: usize, imm: i64 }, // RV64

	// ---- Branches: if (rs1 <cmp> rs2) pc += offset ----
	Beq  { rs1: usize, rs2: usize, offset: i64 },
	Bne  { rs1: usize, rs2: usize, offset: i64 },
	Blt  { rs1: usize, rs2: usize, offset: i64 },
	Bge  { rs1: usize, rs2: usize, offset: i64 },
	Bltu { rs1: usize, rs2: usize, offset: i64 },
	Bgeu { rs1: usize, rs2: usize, offset: i64 },

	// ---- Jumps ----
	Jal  { rd: usize, offset: i64 },
	Jalr { rd: usize, rs1: usize, imm: i64 },

	// ---- Upper immediates (imm is the finished value, already shifted left by 12) ----
	Lui   { rd: usize, imm: i64 },
	Auipc { rd: usize, imm: i64 },

	// ---- RV64: 32-bit math, result sign-extended to 64 bits ----
	Addiw { rd: usize, rs1: usize, imm: i64 },
	Slliw { rd: usize, rs1: usize, shamt: u32 },
	Srliw { rd: usize, rs1: usize, shamt: u32 },
	Sraiw { rd: usize, rs1: usize, shamt: u32 },
	Addw  { rd: usize, rs1: usize, rs2: usize },
	Subw  { rd: usize, rs1: usize, rs2: usize },
	Sllw  { rd: usize, rs1: usize, rs2: usize },
	Srlw  { rd: usize, rs1: usize, rs2: usize },
	Sraw  { rd: usize, rs1: usize, rs2: usize },

	// ---- System ----
	Ecall,
	Ebreak,
	Fence,

	Illegal(u32),
}

//_____________________-helpers__________________________
fn sign_extend(value: u32, bits: u32) -> i64 {
	let shift = 64 - bits;
	((value as i64) << shift) >> shift
}

// Reads `size` bytes (1, 2, 4 or 8) from memory, little-endian.
fn read(mem: &[u8], addr: u64, size: usize) -> u64 {
	let a = addr as usize;
	let mut bytes = [0u8; 8];
	bytes[..size].copy_from_slice(&mem[a..a + size]);
	u64::from_le_bytes(bytes)
}

// Writes the lowest `size` bytes (1, 2, 4 or 8) of `value` to memory, little-endian.
fn write(mem: &mut [u8], addr: u64, size: usize, value: u64) {
	let a = addr as usize;
	mem[a..a + size].copy_from_slice(&value.to_le_bytes()[..size]);
}

impl Instruction {
	pub fn fetch_instruction(mem: &[u8], pc: u64) -> u32 {
		read(mem, pc, 4) as u32
	}

	pub fn decode_instruction(instr: u32) -> Instruction {
		// These fields sit in the same bits in every format, so pull them out once.
		// An instruction that doesn't use one of them just ignores it.
		let opcode = instr & 0x7F;
		let rd = ((instr >> 7) & 0x1F) as usize;
		let funct3 = (instr >> 12) & 0x7;
		let rs1 = ((instr >> 15) & 0x1F) as usize;
		let rs2 = ((instr >> 20) & 0x1F) as usize;
		let funct7 = instr >> 25;
		// I-type immediate: used by arithmetic, loads, jalr and addiw
		let imm_i = sign_extend(instr >> 20, 12);

		match opcode {
			// ---------- R-type: ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND ----------
			0b0110011 => match (funct3, funct7) {
				(0x0, 0x00) => Instruction::Add { rd, rs1, rs2 },
				(0x0, 0x20) => Instruction::Sub { rd, rs1, rs2 },
				(0x1, 0x00) => Instruction::Sll { rd, rs1, rs2 },
				(0x2, 0x00) => Instruction::Slt { rd, rs1, rs2 },
				(0x3, 0x00) => Instruction::Sltu { rd, rs1, rs2 },
				(0x4, 0x00) => Instruction::Xor { rd, rs1, rs2 },
				(0x5, 0x00) => Instruction::Srl { rd, rs1, rs2 },
				(0x5, 0x20) => Instruction::Sra { rd, rs1, rs2 },
				(0x6, 0x00) => Instruction::Or { rd, rs1, rs2 },
				(0x7, 0x00) => Instruction::And { rd, rs1, rs2 },
				_ => Instruction::Illegal(instr),
			},

			// ---------- I-type arithmetic: ADDI, SLTI, ... SLLI, SRLI, SRAI ----------
			0b0010011 => {
				// RV64: shift amounts go up to 63, so shamt is 6 bits (25:20)
				// and the bits above it (31:26) are called funct6.
				let shamt = (instr >> 20) & 0x3F;
				let funct6 = instr >> 26;

				match funct3 {
					0x0 => Instruction::Addi { rd, rs1, imm: imm_i },
					0x2 => Instruction::Slti { rd, rs1, imm: imm_i },
					0x3 => Instruction::Sltiu { rd, rs1, imm: imm_i },
					0x4 => Instruction::Xori { rd, rs1, imm: imm_i },
					0x6 => Instruction::Ori { rd, rs1, imm: imm_i },
					0x7 => Instruction::Andi { rd, rs1, imm: imm_i },
					0x1 if funct6 == 0x00 => Instruction::Slli { rd, rs1, shamt },
					0x5 if funct6 == 0x00 => Instruction::Srli { rd, rs1, shamt },
					0x5 if funct6 == 0x10 => Instruction::Srai { rd, rs1, shamt },
					_ => Instruction::Illegal(instr),
				}
			}

			// ---------- I-type loads: LB, LH, LW, LD, LBU, LHU, LWU ----------
			0b0000011 => match funct3 {
				0x0 => Instruction::Lb { rd, rs1, imm: imm_i },
				0x1 => Instruction::Lh { rd, rs1, imm: imm_i },
				0x2 => Instruction::Lw { rd, rs1, imm: imm_i },
				0x3 => Instruction::Ld { rd, rs1, imm: imm_i },
				0x4 => Instruction::Lbu { rd, rs1, imm: imm_i },
				0x5 => Instruction::Lhu { rd, rs1, imm: imm_i },
				0x6 => Instruction::Lwu { rd, rs1, imm: imm_i },
				_ => Instruction::Illegal(instr),
			},

			// ---------- S-type stores: SB, SH, SW, SD ----------
			0b0100011 => {
				// imm[11:5] is in bits 31:25, imm[4:0] is in bits 11:7
				let imm = sign_extend(((instr >> 25) << 5) | ((instr >> 7) & 0x1F), 12);

				match funct3 {
					0x0 => Instruction::Sb { rs1, rs2, imm },
					0x1 => Instruction::Sh { rs1, rs2, imm },
					0x2 => Instruction::Sw { rs1, rs2, imm },
					0x3 => Instruction::Sd { rs1, rs2, imm },
					_ => Instruction::Illegal(instr),
				}
			}

			// ---------- B-type branches: BEQ, BNE, BLT, BGE, BLTU, BGEU ----------
			0b1100011 => {
				let imm12 = (instr >> 31) & 0x1;
				let imm11 = (instr >> 7) & 0x1;
				let imm10_5 = (instr >> 25) & 0x3F;
				let imm4_1 = (instr >> 8) & 0xF;
				let offset = sign_extend((imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1), 13);

				match funct3 {
					0x0 => Instruction::Beq { rs1, rs2, offset },
					0x1 => Instruction::Bne { rs1, rs2, offset },
					0x4 => Instruction::Blt { rs1, rs2, offset },
					0x5 => Instruction::Bge { rs1, rs2, offset },
					0x6 => Instruction::Bltu { rs1, rs2, offset },
					0x7 => Instruction::Bgeu { rs1, rs2, offset },
					_ => Instruction::Illegal(instr),
				}
			}

			// ---------- JAL (J-type) ----------
			0b1101111 => {
				let imm20 = (instr >> 31) & 0x1;
				let imm19_12 = (instr >> 12) & 0xFF;
				let imm11 = (instr >> 20) & 0x1;
				let imm10_1 = (instr >> 21) & 0x3FF;
				let offset = sign_extend((imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1), 21);

				Instruction::Jal { rd, offset }
			}

			// ---------- JALR (I-type) ----------
			0b1100111 => match funct3 {
				0x0 => Instruction::Jalr { rd, rs1, imm: imm_i },
				_ => Instruction::Illegal(instr),
			},

			// ---------- U-type: LUI, AUIPC ----------
			// The immediate is bits 31:12 with zeros below; `as i32 as i64` sign-extends it.
			0b0110111 => Instruction::Lui { rd, imm: (instr & 0xFFFF_F000) as i32 as i64 },
			0b0010111 => Instruction::Auipc { rd, imm: (instr & 0xFFFF_F000) as i32 as i64 },

			// ---------- RV64: ADDIW, SLLIW, SRLIW, SRAIW ----------
			0b0011011 => {
				// the W shifts only shift a 32-bit value, so shamt stays 5 bits
				let shamt = (instr >> 20) & 0x1F;

				match (funct3, funct7) {
					(0x0, _) => Instruction::Addiw { rd, rs1, imm: imm_i },
					(0x1, 0x00) => Instruction::Slliw { rd, rs1, shamt },
					(0x5, 0x00) => Instruction::Srliw { rd, rs1, shamt },
					(0x5, 0x20) => Instruction::Sraiw { rd, rs1, shamt },
					_ => Instruction::Illegal(instr),
				}
			}

			// ---------- RV64: ADDW, SUBW, SLLW, SRLW, SRAW ----------
			0b0111011 => match (funct3, funct7) {
				(0x0, 0x00) => Instruction::Addw { rd, rs1, rs2 },
				(0x0, 0x20) => Instruction::Subw { rd, rs1, rs2 },
				(0x1, 0x00) => Instruction::Sllw { rd, rs1, rs2 },
				(0x5, 0x00) => Instruction::Srlw { rd, rs1, rs2 },
				(0x5, 0x20) => Instruction::Sraw { rd, rs1, rs2 },
				_ => Instruction::Illegal(instr),
			},

			// ---------- FENCE ----------
			0b0001111 => match funct3 {
				0x0 => Instruction::Fence,
				_ => Instruction::Illegal(instr),
			},

			// ---------- SYSTEM: ECALL, EBREAK ----------
			0b1110011 => match (funct3, instr >> 20) {
				(0x0, 0x0) => Instruction::Ecall,
				(0x0, 0x1) => Instruction::Ebreak,
				_ => Instruction::Illegal(instr),
			},

			_ => Instruction::Illegal(instr),
		}
	}


    pub fn execute_instruction(x: &mut [u64; 32], mem: &mut [u8], pc: &mut u64, instr: Instruction) -> bool {
		let cur_pc: u64 = *pc;
		let mut next_pc: u64 = cur_pc.wrapping_add(4);
		let mut halt = false;

		// Two casting rules used everywhere below:
		//   imm as u64          an i64 keeps its bits, so -1 becomes 0xFFFF_FFFF_FFFF_FFFF
		//   value as i32 as u64 "take the low 32 bits as a signed number, then widen to 64"

		// No `_ =>` arm on purpose: if you add a variant to the enum and forget
		// to handle it here, the compiler gives an error.
		match instr {
			// ---------- R-type ----------
			// RV64: shift amounts are the low 6 bits (0..63), signed compares use i64
			Instruction::Add { rd, rs1, rs2 } => x[rd] = x[rs1].wrapping_add(x[rs2]),
			Instruction::Sub { rd, rs1, rs2 } => x[rd] = x[rs1].wrapping_sub(x[rs2]),
			Instruction::Sll { rd, rs1, rs2 } => x[rd] = x[rs1] << (x[rs2] & 0x3F),
			Instruction::Slt { rd, rs1, rs2 } => x[rd] = ((x[rs1] as i64) < (x[rs2] as i64)) as u64,
			Instruction::Sltu { rd, rs1, rs2 } => x[rd] = (x[rs1] < x[rs2]) as u64,
			Instruction::Xor { rd, rs1, rs2 } => x[rd] = x[rs1] ^ x[rs2],
			Instruction::Srl { rd, rs1, rs2 } => x[rd] = x[rs1] >> (x[rs2] & 0x3F),
			Instruction::Sra { rd, rs1, rs2 } => x[rd] = ((x[rs1] as i64) >> (x[rs2] & 0x3F)) as u64,
			Instruction::Or { rd, rs1, rs2 } => x[rd] = x[rs1] | x[rs2],
			Instruction::And { rd, rs1, rs2 } => x[rd] = x[rs1] & x[rs2],

			// ---------- I-type arithmetic ----------
			Instruction::Addi { rd, rs1, imm } => x[rd] = x[rs1].wrapping_add(imm as u64),
			Instruction::Slti { rd, rs1, imm } => x[rd] = ((x[rs1] as i64) < imm) as u64,
			Instruction::Sltiu { rd, rs1, imm } => x[rd] = (x[rs1] < imm as u64) as u64,
			Instruction::Xori { rd, rs1, imm } => x[rd] = x[rs1] ^ (imm as u64),
			Instruction::Ori { rd, rs1, imm } => x[rd] = x[rs1] | (imm as u64),
			Instruction::Andi { rd, rs1, imm } => x[rd] = x[rs1] & (imm as u64),
			Instruction::Slli { rd, rs1, shamt } => x[rd] = x[rs1] << shamt,
			Instruction::Srli { rd, rs1, shamt } => x[rd] = x[rs1] >> shamt,
			Instruction::Srai { rd, rs1, shamt } => x[rd] = ((x[rs1] as i64) >> shamt) as u64,

			// ---------- Loads ----------
			// signed loads sign-extend to 64 bits, the "u" versions fill with zeros
			Instruction::Lb { rd, rs1, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				x[rd] = read(mem, addr, 1) as i8 as u64;
			}
			Instruction::Lh { rd, rs1, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				x[rd] = read(mem, addr, 2) as i16 as u64;
			}
			Instruction::Lw { rd, rs1, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				x[rd] = read(mem, addr, 4) as i32 as u64;
			}
			Instruction::Ld { rd, rs1, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				x[rd] = read(mem, addr, 8);
			}
			Instruction::Lbu { rd, rs1, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				x[rd] = read(mem, addr, 1);
			}
			Instruction::Lhu { rd, rs1, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				x[rd] = read(mem, addr, 2);
			}
			Instruction::Lwu { rd, rs1, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				x[rd] = read(mem, addr, 4);
			}

			// ---------- Stores ----------
			// `write` only stores the lowest `size` bytes of the register
			Instruction::Sb { rs1, rs2, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				write(mem, addr, 1, x[rs2]);
			}
			Instruction::Sh { rs1, rs2, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				write(mem, addr, 2, x[rs2]);
			}
			Instruction::Sw { rs1, rs2, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				write(mem, addr, 4, x[rs2]);
			}
			Instruction::Sd { rs1, rs2, imm } => {
				let addr = x[rs1].wrapping_add(imm as u64);
				write(mem, addr, 8, x[rs2]);
			}

			// ---------- Branches ----------
			Instruction::Beq { rs1, rs2, offset } => {
				if x[rs1] == x[rs2] {
					next_pc = cur_pc.wrapping_add(offset as u64);
				}
			}
			Instruction::Bne { rs1, rs2, offset } => {
				if x[rs1] != x[rs2] {
					next_pc = cur_pc.wrapping_add(offset as u64);
				}
			}
			Instruction::Blt { rs1, rs2, offset } => {
				if (x[rs1] as i64) < (x[rs2] as i64) {
					next_pc = cur_pc.wrapping_add(offset as u64);
				}
			}
			Instruction::Bge { rs1, rs2, offset } => {
				if (x[rs1] as i64) >= (x[rs2] as i64) {
					next_pc = cur_pc.wrapping_add(offset as u64);
				}
			}
			Instruction::Bltu { rs1, rs2, offset } => {
				if x[rs1] < x[rs2] {
					next_pc = cur_pc.wrapping_add(offset as u64);
				}
			}
			Instruction::Bgeu { rs1, rs2, offset } => {
				if x[rs1] >= x[rs2] {
					next_pc = cur_pc.wrapping_add(offset as u64);
				}
			}

			// ---------- Jumps ----------
			Instruction::Jal { rd, offset } => {
				x[rd] = cur_pc.wrapping_add(4);
				next_pc = cur_pc.wrapping_add(offset as u64);
			}
			Instruction::Jalr { rd, rs1, imm } => {
				// compute the target BEFORE writing rd, because rd and rs1 can be the same register
				let target = x[rs1].wrapping_add(imm as u64) & !1;
				x[rd] = cur_pc.wrapping_add(4);
				next_pc = target;
			}

			// ---------- Upper immediates ----------
			Instruction::Lui { rd, imm } => x[rd] = imm as u64,
			Instruction::Auipc { rd, imm } => x[rd] = cur_pc.wrapping_add(imm as u64),

			// ---------- RV64: 32-bit math ----------
			// cut the inputs to 32 bits, do the math, then sign-extend the result
			Instruction::Addiw { rd, rs1, imm } => x[rd] = (x[rs1] as u32).wrapping_add(imm as u32) as i32 as u64,
			Instruction::Slliw { rd, rs1, shamt } => x[rd] = ((x[rs1] as u32) << shamt) as i32 as u64,
			Instruction::Srliw { rd, rs1, shamt } => x[rd] = ((x[rs1] as u32) >> shamt) as i32 as u64,
			Instruction::Sraiw { rd, rs1, shamt } => x[rd] = ((x[rs1] as i32) >> shamt) as u64,
			Instruction::Addw { rd, rs1, rs2 } => x[rd] = (x[rs1] as u32).wrapping_add(x[rs2] as u32) as i32 as u64,
			Instruction::Subw { rd, rs1, rs2 } => x[rd] = (x[rs1] as u32).wrapping_sub(x[rs2] as u32) as i32 as u64,
			Instruction::Sllw { rd, rs1, rs2 } => x[rd] = ((x[rs1] as u32) << (x[rs2] & 0x1F)) as i32 as u64,
			Instruction::Srlw { rd, rs1, rs2 } => x[rd] = ((x[rs1] as u32) >> (x[rs2] & 0x1F)) as i32 as u64,
			Instruction::Sraw { rd, rs1, rs2 } => x[rd] = ((x[rs1] as i32) >> (x[rs2] & 0x1F)) as u64,

			// ---------- System ----------
			Instruction::Ecall => halt = true,
			Instruction::Ebreak => {
				println!(concat!(
					"Debugger\n",
					"q -- quit\n",
					"<len> @ <addr> -- print <len> bytes at address <addr>\n",
					"<reg> -- print register of name <reg> (including pc)"
				));
				loop {
					eprint!("> ");
					let mut str = String::new();
					if io::stdin().read_line(&mut str).is_err() {
						break;
					}
					let words: Vec<&str> = str.split_whitespace().collect();
					match words.as_slice() {
						&[s_len, "@", s_addr] => {
							let len = s_len.parse::<usize>();
							let addr = s_addr.parse::<usize>();
							match (len, addr) {
								(Ok(L), Ok(A)) => println!("{:?}", &mem[A..A + L]),
								_ => println!("bad range."),
							}
						}
						&["q"] => break,
						&[reg] => match asm::str2reg(&reg.to_string()) {
							Some(n) => println!("{} = {}", n, x[n as usize]),
							_ => eprintln!("bad register."),
						},
						&[] => {}
						_ => eprintln!("bad command."),
					}
				}
			}
			Instruction::Fence => {} // nothing to do in a simple single-core emulator

			Instruction::Illegal(raw) => {
				panic!("illegal instruction {:#010x} at pc {:#x}", raw, cur_pc);
			}
		}

		x[0] = 0; // x0 is always zero, even if an instruction wrote to it
		*pc = next_pc;
		halt
	}
}

