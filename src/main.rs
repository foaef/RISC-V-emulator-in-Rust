mod runner;

fn main() {
	// machien code buffer
	let buffer = Vec::<u8>::new();
	
	//registers
	let mut pc: u32 = 0;
	let mut x: [u32; 32] = [0; 32];
	let mut f: [u32; 32] = [0; 32];

	while (pc as usize) < buffer.len(){
		let mut instruction_len: usize = 0;
		let cur_intsruction: u32 = runner::fetch_instruction(&buffer, &mut pc, &mut instruction_len);

		runner::decode_intruction(&cur_intsruction, &instruction_len);
		runner::execute_instruction(); //input egal whatever cacat are nevoie nush inca
	}
}
