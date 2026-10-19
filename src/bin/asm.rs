use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
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
		if instr.label.is_none() && instr.name.is_none() {
			continue
		}
		if instr.label.is_some() {
			symtab.insert(instr.label.clone().unwrap(), pc);
		}
		if instr.name.is_some() {
			pc += 4;
		}
		instrs.push(instr);
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
