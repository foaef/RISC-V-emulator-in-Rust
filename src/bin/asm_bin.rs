use RISC_V_emulator_in_Rust::asm::assemble;
use std::fs::File;
use std::io::{stdin, BufReader};
use std::env::args;

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
