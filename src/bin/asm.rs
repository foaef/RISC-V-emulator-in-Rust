use std::fs::File;
use std::io::{stdin, BufRead, BufReader};
use std::env::args;

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
				// scur.push(c);
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
		} else if c == ';' || c == '#' {
			break
		} else if c == ',' || c == '(' || c == ')' || c == ':' {
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

fn lexed2instr(vec: &Vec<String>) {
	let mut v: Vec<&str> = vec![];
	for x in vec {
		v.push(x.as_str())
	}
	match &v[..] {
		[label, ":"] => {
			println!("eticheta")
		},
		[instr, arg1, ",", arg2, ",", arg3] => {
			println!("instr 3 arg")
		},
		[instr, arg1, ",", arg2] => {
			println!("instr 2 arg")
		},
		[instr] => {
			println!("instr 1 arg")
		},
		_ => {
			println!("?")
		}
	}
	// wip
}

fn assemble(reader: impl BufRead) {
	for line in reader.lines() {
		lexed2instr(&lex(&line.unwrap()));
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
