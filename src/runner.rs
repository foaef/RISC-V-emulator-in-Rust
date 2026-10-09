pub fn fetch_instruction(v: &Vec<u8>, pc: &mut u32, instruction_len: &mut usize) -> u32{

		/* 
			if first byte read has last 2 bits = 1
				instruction is 4 bytes long
			else
				instruction is 2 bytes long
		*/

		let mut instruction: u32 = v[*pc as usize].into();
		//0b11 is a bitmask: 0..011 checks if last 2 bits are both 1 or not
		let lun = if instruction & 0b11 == 0b11 { 4 } else if instruction & 1 == 1 { 2 } else {
			panic!("instruction invalid: last bit 0");
		};
		if *pc as usize + lun > v.len() {
			panic!("instruction too short");
		}
		for i in 1..lun {
			//building instruction using little-endian
			instruction |= (v[i + *pc as usize] as u32) << 8 * i;
		}
		*instruction_len = lun;
		return instruction;
}

pub fn decode_intruction(instruction: &u32, instruction_len: &usize) {
	//intruction decoding logic and parameters for opcode and whatnot
	//maybe make a struct for exe_instr and return that here?
}

//TBD
pub fn execute_instruction(){

}
		