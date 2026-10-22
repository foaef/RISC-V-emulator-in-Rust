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
			//R-Type
			0b00110011 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.rs2 = ((instr_copy & 0x1F00000) >> 20 )as u8;
				instr.funct7 = ((instr_copy & 0xFE000000) >> 25) as u8;
			}

			//I-type
			0b00010011 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.imm = (instr_copy >> 20) as i32;
				//rs2 si funct7 nu exista
								
				//this sign extends the immediate
				instr.imm = instr.imm << 20;
				instr.imm = instr.imm >> 20;
			}
			//I-type with different opcode
			0b11 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.imm = (instr_copy >> 20) as i32;

				//this sign extends the immediate
				instr.imm = instr.imm << 20;
				instr.imm = instr.imm >> 20;
			}
			//I-type jalr
			0b01100111 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.imm = (instr_copy >> 20) as i32;

				//this sign extends the immediate
				instr.imm = instr.imm << 20;
				instr.imm = instr.imm >> 20;
			}
			//I-type ecall ebreak
			0b01110011 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.imm = (instr_copy >> 20) as i32;

				//this sign extends the immediate
				instr.imm = instr.imm << 20;
				instr.imm = instr.imm >> 20;
			}

			//S-type
			0b00100011 => {
				instr.imm = ((instr_copy & 0xF80) >> 7) as i32;
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.rs2 = ((instr_copy & 0x1F00000) >> 20 )as u8;
				instr.imm |= ((instr_copy >> 25) as i32) >> 21;

				//this sign extends the immediate
				instr.imm = instr.imm << 20;
				instr.imm = instr.imm >> 20;
			}

			//B-type
			0b01100011 => {
				instr.funct3 = ((instr_copy & 0x7000) >> 12) as u8;
				instr.rs1 = ((instr_copy & 0xF8000) >> 15) as u8;
				instr.rs2 = ((instr_copy & 0x1F00000) >> 20 )as u8;

				//decoding and reconstructing immediate bullshit ahh
				let imm12   = (instr_copy >> 31) & 0x1;    // Bit 31
				let imm11   = (instr_copy >> 7)  & 0x1;    // Bit 7
				let imm10_5 = (instr_copy >> 25) & 0x3F;   // Bits 30:25 (6 bits)
				let imm4_1  = (instr_copy >> 8)  & 0xF;    // Bits 11:8  (4 bits)
				// imm[0] is always implicitly 0 

				let raw = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);

				instr.imm = ((raw << 19) as i32) >> 19;
			}

			//U-type
			0b00110111 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.imm = ((instr_copy & 0xFFFFF000) >> 12) as i32;

				//this sign extends the immediate
				instr.imm = instr.imm << 20;
				instr.imm = instr.imm >> 20;
			}
			//U-type diff opcode
			0b00010111 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;
				instr.imm = ((instr_copy & 0xFFFFF000) >> 12) as i32;

				//this sign extends the immediate
				instr.imm = instr.imm << 20;
				instr.imm = instr.imm >> 20;
			}

			//J-type
			0b01101111 => {
				instr.rd = ((instr_copy & 0xF80) >> 7) as u8;

				//decode BULLSHIT AHH immediate
				let imm20    = (instr_copy >> 31) & 0x1;       // Bit 31 (1 bit)
				let imm10_1  = (instr_copy >> 21) & 0x3FF;     // Bits 30:21 (10 bits, mask 0x3FF = 0b1111111111)
				let imm11    = (instr_copy >> 20) & 0x1;       // Bit 20 (1 bit)
				let imm19_12 = (instr_copy >> 12) & 0xFF;      // Bits 19:12 (8 bits, mask 0xFF = 0b11111111)
				// imm[0] is always implicitly 0

				let raw = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);

				instr.imm = ((raw << 11) as i32) >> 11;
			}
			_ => {}
		}
		
	}

	//TBD
//TBD
    pub fn execute_instruction(x: &mut [u32; 32], funct3: u32, funct7: u32, rd: usize, rs1: usize, rs2: usize, imm: i32, opcode: u8) {
        match opcode {
            // ADD-SLTU
            0b0110011 => {
                match (funct3, funct7) {
                    // ADD
                    (0x0, 0x00) => {
                        x[rd] = x[rs1].wrapping_add(x[rs2]);
                    }
                    // SUB
                    (0x0, 0x20) => {
                        x[rd] = x[rs1].wrapping_sub(x[rs2]);
                    }
                    // XOR
                    (0x4, 0x00) => {
                        x[rd] = x[rs1] ^ x[rs2];
                    }
                    // OR
                    (0x6, 0x00) => {
                        x[rd] = x[rs1] | x[rs2];
                    }
                    // AND
                    (0x7, 0x00) => {
                        x[rd] = x[rs1] & x[rs2];
                    }
                    // SLL
                    (0x1, 0x00) => {
                        x[rd] = x[rs1] << (x[rs2] & 0x1F);
                    }
                    // SRL
                    (0x5, 0x00) => {
                        x[rd] = x[rs1] >> (x[rs2] & 0x1F);
                    }
                    // SRA
                    (0x5, 0x20) => {
                        x[rd] = ((x[rs1] as i32) >> (x[rs2] & 0x1F)) as u32;
                    }
                    // SLT 
                    (0x2, 0x00) => {
                        x[rd] = if (x[rs1] as i32) < (x[rs2] as i32) { 1 } else { 0 };
                    }
                    // SLTU
                    (0x3, 0x00) => {
                        x[rd] = if x[rs1] < x[rs2] { 1 } else { 0 };
                    }
                    _ => {}
                }
            }

            // ADDI-SLTIU
            0b0010011 => {
                match (funct3, (imm >> 5) & 0x7F) {
                    // ADDI
                    (0x0, _) => {
                        x[rd] = x[rs1].wrapping_add(imm as u32);
                    }
                    // XORI
                    (0x4, _) => {
                        x[rd] = x[rs1] ^ (imm as u32);
                    }
                    // ORI
                    (0x6, _) => {
                        x[rd] = x[rs1] | (imm as u32);
                    }
                    // ANDI
                    (0x7, _) => {
                        x[rd] = x[rs1] & (imm as u32);
                    }
                    // SLLI
                    (0x1, 0x00) => {
                        x[rd] = x[rs1] << (imm & 0x1F);
                    }
                    // SRLI
                    (0x5, 0x00) => {
                        x[rd] = x[rs1] >> (imm & 0x1F);
                    }
                    // SRAI
                    (0x5, 0x20) => {
                        x[rd] = ((x[rs1] as i32) >> (imm & 0x1F)) as u32;
                    }
                    // SLTI
                    (0x2, _) => {
                        x[rd] = if (x[rs1] as i32) < imm { 1 } else { 0 };
                    }
                    // SLTIU
                    (0x3, _) => {
                        x[rd] = if x[rs1] < (imm as u32) { 1 } else { 0 };
                    }
                    _ => {}
                }
            }

            _ => {}
        }
        x[0] = 0;
    }
} 
