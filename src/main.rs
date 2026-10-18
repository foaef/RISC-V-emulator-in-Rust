mod cpu;

fn main() {

	while (pc as usize) < buffer.len(){
		let mut instruction_len: usize = 0;
		let cur_intsruction: u32 = runner::fetch_instruction(&buffer, &mut pc, &mut instruction_len);

		runner::decode_intruction(&cur_intsruction, &instruction_len);
		runner::execute_instruction(); //input egal whatever cacat are nevoie nush inca
	}
}
