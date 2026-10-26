use std::assert_matches;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write, stdin, stdout};
use std::env::args;

#[derive(Debug)]
pub enum Operand {
	Direct(String),
	Indirect { offset: String, base: String }
}

#[derive(Debug)]
pub struct AsmInstr {
	pub label: Option<String>,
	pub name: Option<String>,
	pub args: Vec<Operand>
}

fn lex(s: &str) -> Vec<String> {
	let mut rv: Vec<String> = vec![];
	let mut scur: String = Default::default();
	let mut in_str = false;
	let bytes = s.as_bytes();
	let mut i = 0;
	while i < bytes.len() {
		let c = bytes[i] as char;
		if in_str {
			if c == '"' {
				scur.push(c);
				in_str = false;
				rv.push(scur.clone());
				scur.clear();
			} else if c == '\\' && i < bytes.len() - 1 {
				scur.push(bytes[i + 1] as char);
				i += 2;
				continue;
			} else {
				scur.push(c);
			}

		} else if c == '"' {
			if !scur.is_empty() {
				rv.push(scur.clone());
				scur.clear()
			}
			scur.push(c);
			in_str = true;
		} else if ";#".contains(c) {
			break
		} else if ",:()".contains(c) {
			if !scur.is_empty() {
				rv.push(scur.clone());
				scur.clear()
			}
			rv.push(c.to_string())
		} else if c.is_whitespace() {
			if !scur.is_empty() {
				rv.push(scur.clone());
				scur.clear()
			}
		} else {
			scur.push(c);
		}
		i += 1;
	}
	if !scur.is_empty() {
		rv.push(scur)
	}
	rv
}

fn lexed2instr(vec: &[String]) -> Option<AsmInstr> {
	let v: Vec<&str> = vec
		.iter()
		.map(|s| s.as_str())
		.collect();
	let (label, instr_wo_label) = match &v[..] {
		[label, ":", rest @ ..] => (Some(label.to_string()), rest),
		_ => (None, &v[..])
	};
	if instr_wo_label.is_empty() {
		return Some(AsmInstr {
			label,
			name:  None,
			args:  vec![]
		})
	}
	let name = Some(instr_wo_label[0].to_string());
	let mut args: Vec<Operand> = vec![];
	let bucati: Vec<&[&str]> = (&instr_wo_label[1..])
		.split(|s| *s == ",")
		.filter(|s| !s.is_empty())
		.collect();
	for buc in bucati {
		args.push(match buc {
			[base]
				=> Operand::Direct(base.to_string()),
			[offset, "(", base, ")"]
				=> Operand::Indirect {
					offset: offset.to_string(),
					base: base.to_string()
				},
			["(", base, ")"]
				=> Operand::Indirect {
					offset: "".to_string(),
					base: base.to_string()
				},
			_   => return None
		})
	}
	Some(AsmInstr { label, name, args })
}

fn str2reg(s: &String) -> Option<u32> {
	if s == "zero" { return Some(0) }
	if s == "ra"   { return Some(1) }
	if s == "sp"   { return Some(2) }
	if s == "gp"   { return Some(3) }
	if s == "tp"   { return Some(4) }
	if s == "fp"   { return Some(8) }
	let range = |str: &String, c: u8, l: i8, r: i8, L: u8| { 
		let s = str.as_bytes();
		if s.len() >= 1 && s[0] == c {
			let mut n: i8 = -1;
			if s.len() == 2 {
				n = (s[1] - b'0') as i8;
			} else if s.len() == 3 {
				n = ((s[1] - b'0') * 10 + s[2] - b'0') as i8;
			}
			if l <= n && n <= r {
				Some(L + n as u8)
			} else {
				None
			}
		} else {
			None
		}
	};
	let idx = range(s, b'x', 0, 31, 0)
		.or_else(|| range(s, b't', 0, 2, 5))
		.or_else(|| range(s, b't', 3, 6, 25))
		.or_else(|| range(s, b'a', 0, 7, 10))
		.or_else(|| range(s, b's', 0, 1, 8))
		.or_else(|| range(s, b's', 2, 11, 16))
		.or_else(|| range(s, b'f', 0, 31, 0))?;
	Some(idx as u32)
}

fn nth_direct(instr: &AsmInstr, n: usize) -> u32 {
	match &instr.args[n] {
		Operand::Direct(s) => str2reg(s).unwrap(),
		_                  => panic!("bad direct argument")
	}
}

fn nth_direct_imm(instr: &AsmInstr, n: usize) -> i32 {
	match &instr.args[n] {
		Operand::Direct(s) => s.parse::<i32>().unwrap(),
		_                  => panic!("bad imm argument (direct)")
	}
}

fn nth_indirect_imm(instr: &AsmInstr, n: usize) -> (i32, u32) {
	match &instr.args[n] {
		Operand::Indirect { offset, base } => (offset.parse::<i32>().unwrap(),
		                                       str2reg(base).unwrap()),
		_	                               => panic!("bad imm argument (indirect)")
	}
}

fn write_u32(b: u32) {
	let bytes = b.to_be_bytes();
	stdout().write_all(&bytes).unwrap()
}

// toate tipurile de instructiuni
fn emit_r(instr: &AsmInstr, opcode: u32, funct3: u32, funct7: u32) {
	let rd:  u32;
	let rs1: u32;
	let rs2: u32;
	
	// reguli
	assert!(instr.args.len() == 3);
	assert_matches!(instr.args[0], Operand::Direct(_));
	assert_matches!(instr.args[1], Operand::Direct(_));
	assert_matches!(instr.args[2], Operand::Direct(_));
	rd  = nth_direct(instr, 0);
	rs1 = nth_direct(instr, 1);
	rs2 = nth_direct(instr, 2);
	
	write_u32(opcode
		| (rd << 7) | (funct3 << 12)
		| (rs1 << 15) | (rs2 << 20)
		| (funct7 << 25));
}
fn emit_i(instr: &AsmInstr, opcode: u32, funct3: u32, paren: bool, srai: bool) {
	let rd:  u32;
	let rs1: u32;
	let mut imm: i32;

	// reguli
	if paren {
		assert!(instr.args.len() == 2);
		assert_matches!(instr.args[0], Operand::Direct(_));
		assert_matches!(instr.args[1], Operand::Indirect { .. });
		rd  = nth_direct(instr, 0);
		(imm, rs1) = nth_indirect_imm(instr, 1);
	} else {
		assert!(instr.args.len() == 3);
		assert_matches!(instr.args[0], Operand::Direct(_));
		assert_matches!(instr.args[1], Operand::Direct(_));
		assert_matches!(instr.args[2], Operand::Direct(_));
		rd  = nth_direct(instr, 0);
		rs1 = nth_direct(instr, 1);
		imm = nth_direct_imm(instr, 2);
	}

	if srai {
		imm |= 0x20 << 5;
	}
	write_u32(opcode
		| (rd << 7) | (funct3 << 12)
		| (rs1 << 15) | ((imm as u32) << 20));
}
fn emit_s(instr: &AsmInstr, opcode: u32, funct3: u32) {
	let rs1: u32;
	let rs2: u32;
	let mut imm: i32;

	// reguli
	assert!(instr.args.len() == 2);
	assert_matches!(instr.args[0], Operand::Direct(_));
	assert_matches!(instr.args[1], Operand::Indirect { .. });
	rs2 = nth_direct(instr, 0);
	(imm, rs1) = nth_indirect_imm(instr, 1);

	write_u32(opcode
		| (imm as u32 & 0b11111) | (funct3 << 12)
		| (rs1 << 15) | (rs2 << 20) | (imm as u32 & 0b111111100000))
}
fn emit_b(instr: &AsmInstr, opcode: u32, funct3: u32, symtab: &HashMap<String, u32>, pc: u32) {
	let rs1: u32;
	let rs2: u32;
	let imm: i32;

	// reguli
	assert!(instr.args.len() == 3);
	assert_matches!(instr.args[0], Operand::Direct(_));
	assert_matches!(instr.args[1], Operand::Direct(_));
	assert_matches!(instr.args[2], Operand::Direct(_));
	let label = match instr.args[2] {
		Operand::Direct(ref l) => l,
		_ => unreachable!()
	};
	assert!(symtab.contains_key(label));
	rs1 = nth_direct(instr, 0);
	rs2 = nth_direct(instr, 1);
	imm = (pc as i32) - (symtab[label] as i32);

	write_u32(opcode
		| (imm as u32 & (0b100000011110) << 7) | (funct3 << 12)
		| (rs1 << 15) | (rs2 << 20) | (imm as u32 & 0b1011111100000))
}

fn assemble(reader: impl BufRead) {
	let mut instrs: Vec<AsmInstr> = vec![];
	let mut pc: u32 = 0;
	let mut symtab: HashMap<String, u32> = HashMap::new();

	//
	// faza 1
	//
	for line in reader.lines() {
		let lexed = lex(&line.unwrap());
		let Some(instr) = lexed2instr(&lexed) else { continue };
		if instr.label.is_some() {
			symtab.insert(instr.label.clone().unwrap(), pc);
		}
		if instr.name.is_some() {
			pc += 4;
			instrs.push(instr);
		}
	}

	//
	// faza 2
	//
	pc = 0;
	for instr in instrs {
		match instr.name.as_ref().unwrap().as_str() {
			// tip R
			"add"   => emit_r(&instr, 0b0110011, 0x0, 0x00),
			"sub"   => emit_r(&instr, 0b0110011, 0x0, 0x20),
			"sll"   => emit_r(&instr, 0b0110011, 0x4, 0x00),
			"slt"   => emit_r(&instr, 0b0110011, 0x6, 0x00),
			"sltu " => emit_r(&instr, 0b0110011, 0x7, 0x00),
			"xor"   => emit_r(&instr, 0b0110011, 0x1, 0x00),
			"srl"   => emit_r(&instr, 0b0110011, 0x5, 0x00),
			"sra"   => emit_r(&instr, 0b0110011, 0x5, 0x20),
			"or"    => emit_r(&instr, 0b0110011, 0x2, 0x00),
			"and"   => emit_r(&instr, 0b0110011, 0x3, 0x00),
			// tip I
			"addi"  => emit_i(&instr, 0b0010011, 0x0, false, false),
			"xori"  => emit_i(&instr, 0b0010011, 0x4, false, false),
			"ori"   => emit_i(&instr, 0b0010011, 0x6, false, false),
			"andi"  => emit_i(&instr, 0b0010011, 0x7, false, false),
			"slli"  => emit_i(&instr, 0b0010011, 0x1, false, false),
			"srli"  => emit_i(&instr, 0b0010011, 0x5, false, false),
			"srai"  => emit_i(&instr, 0b0010011, 0x5, false,  true),
			"slti"  => emit_i(&instr, 0b0010011, 0x2, false, false),
			"sltiu" => emit_i(&instr, 0b0010011, 0x3, false, false),
			
			"lb"    => emit_i(&instr, 0b0000011, 0x0,  true, false),
			"lh"    => emit_i(&instr, 0b0000011, 0x1,  true, false),
			"lw"    => emit_i(&instr, 0b0000011, 0x2,  true, false),
			"lbu"   => emit_i(&instr, 0b0000011, 0x4,  true, false),
			"lhu"   => emit_i(&instr, 0b0000011, 0x5,  true, false),

			"ecall" => {
				assert!(instr.args.len() == 0);
				write_u32(0x0000073)
			}
			// wip
			// tip S
			"sb"    => emit_s(&instr, 0b0010111, 0x0),
			"sh"    => emit_s(&instr, 0b0010111, 0x1),
			"sw"    => emit_s(&instr, 0b0010111, 0x2),
			// tip B
			"beq"   => emit_b(&instr, 0b0111111, 0x0, &symtab, pc),
			"bne"   => emit_b(&instr, 0b0111111, 0x1, &symtab, pc),
			"blt"   => emit_b(&instr, 0b0111111, 0x4, &symtab, pc),
			"bge"   => emit_b(&instr, 0b0111111, 0x5, &symtab, pc),
			"bltu"  => emit_b(&instr, 0b0111111, 0x6, &symtab, pc),
			"bgtu"  => emit_b(&instr, 0b0111111, 0x7, &symtab, pc),
			
			// necunoscut
			&_     => todo!()
		}
		pc += 4;
	}
}

fn main() {
	if args().len() == 1 {
		assemble(stdin().lock())
	} else {
		for arg in args().skip(1) {
			if arg == "-" {
				assemble(stdin().lock())
			} else {
				assemble(BufReader::new(File::open(&arg).unwrap()))
			}
		}
	}
}
