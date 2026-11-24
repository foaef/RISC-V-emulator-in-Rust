use RISC_V_emulator_in_Rust::cpu::Cpu;
use std::{env, fs, io};

fn boot(file: &String, memsize: usize) -> io::Result<()> {
	let mut mem: Vec<u8> = fs::read(file)?;
	if memsize > mem.len() {
		mem.resize(memsize, 0u8)
	}
	let mut cpu = Cpu::new(mem);
	cpu.run();
	Ok(())
}

fn usage(prog: &String) {
	eprintln!("usage: {} [-m <memsize>] program", prog);
	std::process::exit(1)
}

fn main() {
	let mut parsed_cli: bool = false;
	let mut booted: bool     = false;
	let mut args             = env::args().skip(1);
	let mut memsize: usize   = 1000; // 1kB default
	let prog                 = &env::args().nth(0).unwrap();
	while let Some(arg) = args.next() {
		if !parsed_cli {
			if arg.chars().nth(0) == Some('-') {
				match arg.as_str() {
					"--" => parsed_cli = true,
					"-m" | "--mem" => {
						let val = args.next().expect("no memsize.");
						memsize = val.parse::<usize>().expect("memsize not a number.")
					}
					// etc etc
					_ => usage(prog)
				}
			} else {
				boot(&arg, memsize).unwrap();
				booted = true;
			}
		} else {
			boot(&arg, memsize).unwrap();
			booted = true;
		}
	}
	if !booted {
		usage(prog)
	}
}
