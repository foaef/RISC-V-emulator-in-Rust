use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader};
use std::collections::HashMap;
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
	let range = |str: &String| { move |c: u8, l: i8, r: i8, L: u8| {
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
	}};
	let idx = range(s)(b'x', 0, 31, 0)
		.or_else(|| range(s)(b't', 0, 2, 5))
		.or_else(|| range(s)(b't', 3, 6, 25))
		.or_else(|| range(s)(b'a', 0, 7, 10))
		.or_else(|| range(s)(b's', 0, 1, 8))
		.or_else(|| range(s)(b's', 2, 11, 16))
		.or_else(|| range(s)(b'f', 0, 31, 0))?;
	Some(idx as u32)
}

// toate tipurile de instructiuni
fn emit_R(opcode: u8, rd: u8, funct3: u8, rs1: u8, rs2: u8, funct7: u8) {
	let out: u32 = opcode
		| (rd << 7) | (funct3 << 12)
		| (rs1 << 15) | (rs1 << 20)
		| (funct7 << 25);
	let bytes = out.to_le_bytes();
	io::stdout().write_all(&bytes).unwrap()
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
	for instr in instrs {
		// wip
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
