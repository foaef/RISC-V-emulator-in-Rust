#[path = "../cpu.rs"]
mod cpu;
use crate::cpu::ExeInstr;

fn main() {
	// poate pastram asta momentan doar ca nu merge cargo build fara
	// machien code buffer
	let buffer = Vec::<u8>::new();
	
	//registers
	let mut pc: u32 = 0;
	let mut x: [u32; 32] = [0; 32];
	let mut f: [u32; 32] = [0; 32];

	while (pc as usize) < buffer.len(){
		let mut instruction_len: usize = 0;
		let cur_instruction: u32 = ExeInstr::fetch_instruction(&buffer, &mut pc, &mut instruction_len);

		// ExeInstr::decode_instruction(&mut cur_instruction, &instruction_len);
		// ExeInstr::execute_instruction(); //input egal whatever cacat are nevoie nush inca
	}
}
