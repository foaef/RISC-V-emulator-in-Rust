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
pub fn execute_instruction(x: &mut [u32; 32], funct3: u32, funct7: u32, rd: usize, rs1: usize, rs2: usize) {
    match (funct3, funct7) {
        //ADD
        (0x0, 0x00) => {
            x[rd] = x[rs1].wrapping_add(x[rs2]);
        }
        //SUB
        (0x0, 0x20) => {
            x[rd] = x[rs1].wrapping_sub(x[rs2]);
        }
        //XOR
        (0x4, 0x00) => {
            x[rd] = x[rs1] ^ x[rs2];
        }
        //OR
        (0x6, 0x00) => {
            x[rd] = x[rs1] | x[rs2];
        }
        //AND
        (0x7, 0x00) => {
            x[rd] = x[rs1] & x[rs2];
        }
        //SLL
        (0x1, 0x00) => {
            x[rd] = x[rs1] << (x[rs2] & 0x1F);
        }
        //SRL
        (0x5, 0x00) => {
            x[rd] = x[rs1] >> (x[rs2] & 0x1F);
        }
        //SRA
        (0x5, 0x20) => {
            x[rd] = ((x[rs1] as i32) >> (x[rs2] & 0x1F)) as u32;
        }
        //SLT 
        (0x2, 0x00) => {
            x[rd] = if (x[rs1] as i32) < (x[rs2] as i32) { 1 } else { 0 };
        }
        //SLTU
        (0x3, 0x00) => {
            x[rd] = if x[rs1] < x[rs2] { 1 } else { 0 };
        }
        _ => {}
    }
    x[0]=0;
}
