use core::prelude::rust_2015;

#[derive(Debug)]
pub struct CPU{
	//registers
	x: Vec<u32>,
	f: Vec<u32>,

	//program counter
	pc: u32,

	//Memory
	mem: Vec<u8>
}

#[derive(Debug)]
pub struct ExeInstr{
	instr32: u32,
	instr_len: usize,
	imm: i32,
	opcode: u8,
	rd: u8,
	rs1: u8,
	rs2: u8,
	funct3: u8,
	funct7: u8
}

impl ExeInstr {
	pub fn fetch_instruction(v: &Vec<u8>, pc: &mut u32, instruction_len: &mut usize /* ? ExeInstr::instr_len */) -> u32{

			/* 
				if first byte read has last 2 bits = 1
					instruction is 4 bytes long
				else
					instruction is 2 bytes long
			*/

			let mut instruction: u32 = v[*pc as usize].into();
			//0b11 is a bitmask: 0..011 checks if last 2 bits are both 1 or not
			let lun = if instruction & 0b11 == 0b11 { 4 } else { 2 };
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

	pub fn decode_instruction(instr: &mut ExeInstr, instruction_len: &usize) {
		let instr_copy: u32 = instr.instr32;
		instr.opcode = (instr_copy & 0b01111111) as u8;
		match instr.opcode {
			//B-Type
			0b00110011 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.rs2 = ((instr_copy & 0x1F00000) >> 20 )as u8;
				instr.funct7 = ((instr_copy & 0xFE000000) >> 25) as u8;
			}
			//Restu dupa 
			_ => {}
		}
		
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
}		
