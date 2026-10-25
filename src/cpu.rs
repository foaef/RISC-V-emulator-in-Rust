#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    // ---- R-type: rd = rs1 <op> rs2 ----
    Add  { rd: usize, rs1: usize, rs2: usize },
    Sub  { rd: usize, rs1: usize, rs2: usize },
    Sll  { rd: usize, rs1: usize, rs2: usize },
    Slt  { rd: usize, rs1: usize, rs2: usize },
    Sltu { rd: usize, rs1: usize, rs2: usize },
    Xor  { rd: usize, rs1: usize, rs2: usize },
    Srl  { rd: usize, rs1: usize, rs2: usize },
    Sra  { rd: usize, rs1: usize, rs2: usize },
    Or   { rd: usize, rs1: usize, rs2: usize },
    And  { rd: usize, rs1: usize, rs2: usize },
 
    // ---- I-type arithmetic: rd = rs1 <op> imm ----
    Addi  { rd: usize, rs1: usize, imm: i32 },
    Slti  { rd: usize, rs1: usize, imm: i32 },
    Sltiu { rd: usize, rs1: usize, imm: i32 },
    Xori  { rd: usize, rs1: usize, imm: i32 },
    Ori   { rd: usize, rs1: usize, imm: i32 },
    Andi  { rd: usize, rs1: usize, imm: i32 },
    Slli  { rd: usize, rs1: usize, shamt: u32 },
    Srli  { rd: usize, rs1: usize, shamt: u32 },
    Srai  { rd: usize, rs1: usize, shamt: u32 },
 
    // ---- Loads: rd = mem[rs1 + imm] ----
    Lb  { rd: usize, rs1: usize, imm: i32 },
    Lh  { rd: usize, rs1: usize, imm: i32 },
    Lw  { rd: usize, rs1: usize, imm: i32 },
    Lbu { rd: usize, rs1: usize, imm: i32 },
    Lhu { rd: usize, rs1: usize, imm: i32 },
 
    // ---- Stores: mem[rs1 + imm] = rs2 ----
    Sb { rs1: usize, rs2: usize, imm: i32 },
    Sh { rs1: usize, rs2: usize, imm: i32 },
    Sw { rs1: usize, rs2: usize, imm: i32 },
 
    // ---- Branches: if (rs1 <cmp> rs2) pc += offset ----
    Beq  { rs1: usize, rs2: usize, offset: i32 },
    Bne  { rs1: usize, rs2: usize, offset: i32 },
    Blt  { rs1: usize, rs2: usize, offset: i32 },
    Bge  { rs1: usize, rs2: usize, offset: i32 },
    Bltu { rs1: usize, rs2: usize, offset: i32 },
    Bgeu { rs1: usize, rs2: usize, offset: i32 },
 
    // ---- Jumps ----
    Jal  { rd: usize, offset: i32 },
    Jalr { rd: usize, rs1: usize, imm: i32 },
 
    // ---- Upper immediates ----
    Lui   { rd: usize, imm: i32 },
    Auipc { rd: usize, imm: i32 },
 
    // ---- System ----
    Ecall,
    Ebreak,
    Fence,
 
    Illegal(u32),
}

impl Instruction {
	pub fn fetch_instruction(v: &Vec<u8>, pc: &mut u32, instruction_len: &mut usize /* ? ExeInstr::instr_len */) -> u32{

			/* 
				if first byte read has last 2 bits = 1
					instruction is 4 bytes long
				else
					instruction is 2 bytes long
			*/

			let mut instruction: u32 = v[*pc as usize].into();
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

	pub fn decode_instruction(instr: u32) -> Instruction {
    let opcode = instr & 0x7F;
 
    // First match on the opcode,
    // then extract only the fields that format actually has.
    match opcode {
        // ---------- R-type: ADD, SUB, SLL, SLT, SLTU, XOR, SRL, SRA, OR, AND ----------
        0b0110011 => {
            let rd = ((instr >> 7) & 0x1F) as usize;
            let funct3 = (instr >> 12) & 0x7;
            let rs1 = ((instr >> 15) & 0x1F) as usize;
            let rs2 = ((instr >> 20) & 0x1F) as usize;
            let funct7 = instr >> 25;
 
            match (funct3, funct7) {
                (0x0, 0x00) => Instruction::Add { rd, rs1, rs2 },
                (0x0, 0x20) => Instruction::Sub { rd, rs1, rs2 },
                (0x1, 0x00) => Instruction::Sll { rd, rs1, rs2 },
                (0x2, 0x00) => Instruction::Slt { rd, rs1, rs2 },
                (0x3, 0x00) => Instruction::Sltu { rd, rs1, rs2 },
                (0x4, 0x00) => Instruction::Xor { rd, rs1, rs2 },
                (0x5, 0x00) => Instruction::Srl { rd, rs1, rs2 },
                (0x5, 0x20) => Instruction::Sra { rd, rs1, rs2 },
                (0x6, 0x00) => Instruction::Or { rd, rs1, rs2 },
                (0x7, 0x00) => Instruction::And { rd, rs1, rs2 },
                _ => Instruction::Illegal(instr),
            }
        }
 
        // ---------- I-type arithmetic: ADDI, SLTI, ... SLLI, SRLI, SRAI ----------
        0b0010011 => {
            let rd = ((instr >> 7) & 0x1F) as usize;
            let funct3 = (instr >> 12) & 0x7;
            let rs1 = ((instr >> 15) & 0x1F) as usize;
            // casting to i32 first makes >> copy the sign bit (sign extension)
            let imm = (instr as i32) >> 20;
            // only used by the shift instructions
            let shamt = (instr >> 20) & 0x1F;
            let funct7 = instr >> 25;
 
            match funct3 {
                0x0 => Instruction::Addi { rd, rs1, imm },
                0x2 => Instruction::Slti { rd, rs1, imm },
                0x3 => Instruction::Sltiu { rd, rs1, imm },
                0x4 => Instruction::Xori { rd, rs1, imm },
                0x6 => Instruction::Ori { rd, rs1, imm },
                0x7 => Instruction::Andi { rd, rs1, imm },
                0x1 if funct7 == 0x00 => Instruction::Slli { rd, rs1, shamt },
                0x5 if funct7 == 0x00 => Instruction::Srli { rd, rs1, shamt },
                0x5 if funct7 == 0x20 => Instruction::Srai { rd, rs1, shamt },
                _ => Instruction::Illegal(instr),
            }
        }
 
        // ---------- I-type loads: LB, LH, LW, LBU, LHU ----------
        0b0000011 => {
            let rd = ((instr >> 7) & 0x1F) as usize;
            let funct3 = (instr >> 12) & 0x7;
            let rs1 = ((instr >> 15) & 0x1F) as usize;
            let imm = (instr as i32) >> 20;
 
            match funct3 {
                0x0 => Instruction::Lb { rd, rs1, imm },
                0x1 => Instruction::Lh { rd, rs1, imm },
                0x2 => Instruction::Lw { rd, rs1, imm },
                0x4 => Instruction::Lbu { rd, rs1, imm },
                0x5 => Instruction::Lhu { rd, rs1, imm },
                _ => Instruction::Illegal(instr),
            }
        }
 
        // ---------- S-type stores: SB, SH, SW ----------
        0b0100011 => {
            let funct3 = (instr >> 12) & 0x7;
            let rs1 = ((instr >> 15) & 0x1F) as usize;
            let rs2 = ((instr >> 20) & 0x1F) as usize;
            // imm[11:5] is in bits 31:25 (sign-extended by the i32 shift),
            // imm[4:0] is in bits 11:7
            let imm = (((instr as i32) >> 25) << 5) | (((instr >> 7) & 0x1F) as i32);
 
            match funct3 {
                0x0 => Instruction::Sb { rs1, rs2, imm },
                0x1 => Instruction::Sh { rs1, rs2, imm },
                0x2 => Instruction::Sw { rs1, rs2, imm },
                _ => Instruction::Illegal(instr),
            }
        }
 
        // ---------- B-type branches: BEQ, BNE, BLT, BGE, BLTU, BGEU ----------
        0b1100011 => {
            let funct3 = (instr >> 12) & 0x7;
            let rs1 = ((instr >> 15) & 0x1F) as usize;
            let rs2 = ((instr >> 20) & 0x1F) as usize;
 
            let imm12 = (instr >> 31) & 0x1;
            let imm11 = (instr >> 7) & 0x1;
            let imm10_5 = (instr >> 25) & 0x3F;
            let imm4_1 = (instr >> 8) & 0xF;
            let raw = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
            // 13-bit value: shift up to the top, then arithmetic shift back down
            let offset = ((raw << 19) as i32) >> 19;
 
            match funct3 {
                0x0 => Instruction::Beq { rs1, rs2, offset },
                0x1 => Instruction::Bne { rs1, rs2, offset },
                0x4 => Instruction::Blt { rs1, rs2, offset },
                0x5 => Instruction::Bge { rs1, rs2, offset },
                0x6 => Instruction::Bltu { rs1, rs2, offset },
                0x7 => Instruction::Bgeu { rs1, rs2, offset },
                _ => Instruction::Illegal(instr),
            }
        }
 
        // ---------- JAL (J-type) ----------
        0b1101111 => {
            let rd = ((instr >> 7) & 0x1F) as usize;
 
            let imm20 = (instr >> 31) & 0x1;
            let imm10_1 = (instr >> 21) & 0x3FF;
            let imm11 = (instr >> 20) & 0x1;
            let imm19_12 = (instr >> 12) & 0xFF;
            let raw = (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1);
            // 21-bit value
            let offset = ((raw << 11) as i32) >> 11;
 
            Instruction::Jal { rd, offset }
        }
 
        // ---------- JALR (I-type) ----------
        0b1100111 => {
            let rd = ((instr >> 7) & 0x1F) as usize;
            let funct3 = (instr >> 12) & 0x7;
            let rs1 = ((instr >> 15) & 0x1F) as usize;
            let imm = (instr as i32) >> 20;
 
            if funct3 == 0x0 {
                Instruction::Jalr { rd, rs1, imm }
            } else {
                Instruction::Illegal(instr)
            }
        }
 
        // ---------- U-type: LUI ----------
        0b0110111 => {
            let rd = ((instr >> 7) & 0x1F) as usize;
            // 20-bit value, NOT shifted yet (execute does `<< 12`)
            let imm = (instr >> 12) as i32;
            Instruction::Lui { rd, imm }
        }
 
        // ---------- U-type: AUIPC ----------
        0b0010111 => {
            let rd = ((instr >> 7) & 0x1F) as usize;
            let imm = (instr >> 12) as i32;
            Instruction::Auipc { rd, imm }
        }
 
        // ---------- FENCE ----------
        0b0001111 => {
            let funct3 = (instr >> 12) & 0x7;
            if funct3 == 0x0 {
                Instruction::Fence
            } else {
                Instruction::Illegal(instr)
            }
        }
 
        // ---------- SYSTEM: ECALL, EBREAK ----------
        0b1110011 => {
            let funct3 = (instr >> 12) & 0x7;
            let imm12 = instr >> 20;
 
            match (funct3, imm12) {
                (0x0, 0x0) => Instruction::Ecall,
                (0x0, 0x1) => Instruction::Ebreak,
                _ => Instruction::Illegal(instr),
            }
        }
 
        _ => Instruction::Illegal(instr),
    }
}


    pub fn execute_instruction(x: &mut [u32; 32], mem: &mut Vec<u8>, pc: &mut u32, instr: Instruction) -> bool {
    let cur_pc: u32 = *pc;
    // by default go to the next instruction; jumps and taken branches change this
    let mut next_pc: u32 = cur_pc.wrapping_add(4);
    let mut halt = false;
 
    // No `_ =>` arm on purpose: if you add a variant to the enum and forget
    // to handle it here, the compiler gives an error.
    match instr {
        // ---------- R-type ----------
        Instruction::Add { rd, rs1, rs2 } => x[rd] = x[rs1].wrapping_add(x[rs2]),
        Instruction::Sub { rd, rs1, rs2 } => x[rd] = x[rs1].wrapping_sub(x[rs2]),
        Instruction::Sll { rd, rs1, rs2 } => x[rd] = x[rs1] << (x[rs2] & 0x1F),
        Instruction::Slt { rd, rs1, rs2 } => {
            x[rd] = if (x[rs1] as i32) < (x[rs2] as i32) { 1 } else { 0 };
        }
        Instruction::Sltu { rd, rs1, rs2 } => {
            x[rd] = if x[rs1] < x[rs2] { 1 } else { 0 };
        }
        Instruction::Xor { rd, rs1, rs2 } => x[rd] = x[rs1] ^ x[rs2],
        Instruction::Srl { rd, rs1, rs2 } => x[rd] = x[rs1] >> (x[rs2] & 0x1F),
        Instruction::Sra { rd, rs1, rs2 } => {
            x[rd] = ((x[rs1] as i32) >> (x[rs2] & 0x1F)) as u32;
        }
        Instruction::Or { rd, rs1, rs2 } => x[rd] = x[rs1] | x[rs2],
        Instruction::And { rd, rs1, rs2 } => x[rd] = x[rs1] & x[rs2],
 
        // ---------- I-type arithmetic ----------
        Instruction::Addi { rd, rs1, imm } => x[rd] = x[rs1].wrapping_add(imm as u32),
        Instruction::Slti { rd, rs1, imm } => {
            x[rd] = if (x[rs1] as i32) < imm { 1 } else { 0 };
        }
        Instruction::Sltiu { rd, rs1, imm } => {
            x[rd] = if x[rs1] < (imm as u32) { 1 } else { 0 };
        }
        Instruction::Xori { rd, rs1, imm } => x[rd] = x[rs1] ^ (imm as u32),
        Instruction::Ori { rd, rs1, imm } => x[rd] = x[rs1] | (imm as u32),
        Instruction::Andi { rd, rs1, imm } => x[rd] = x[rs1] & (imm as u32),
        Instruction::Slli { rd, rs1, shamt } => x[rd] = x[rs1] << shamt,
        Instruction::Srli { rd, rs1, shamt } => x[rd] = x[rs1] >> shamt,
        Instruction::Srai { rd, rs1, shamt } => {
            x[rd] = ((x[rs1] as i32) >> shamt) as u32;
        }
 
        // ---------- Loads ----------
        Instruction::Lb { rd, rs1, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            x[rd] = mem[addr] as i8 as i32 as u32; // sign-extend
        }
        Instruction::Lh { rd, rs1, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            x[rd] = u16::from_le_bytes([mem[addr], mem[addr + 1]]) as i16 as i32 as u32;
        }
        Instruction::Lw { rd, rs1, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            x[rd] = u32::from_le_bytes([mem[addr], mem[addr + 1], mem[addr + 2], mem[addr + 3]]);
        }
        Instruction::Lbu { rd, rs1, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            x[rd] = mem[addr] as u32; // zero-extend
        }
        Instruction::Lhu { rd, rs1, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            x[rd] = u16::from_le_bytes([mem[addr], mem[addr + 1]]) as u32;
        }
 
        // ---------- Stores ----------
        Instruction::Sb { rs1, rs2, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            mem[addr] = x[rs2] as u8;
        }
        Instruction::Sh { rs1, rs2, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            mem[addr..addr + 2].copy_from_slice(&(x[rs2] as u16).to_le_bytes());
        }
        Instruction::Sw { rs1, rs2, imm } => {
            let addr = x[rs1].wrapping_add(imm as u32) as usize;
            mem[addr..addr + 4].copy_from_slice(&x[rs2].to_le_bytes());
        }
 
        // ---------- Branches ----------
        Instruction::Beq { rs1, rs2, offset } => {
            if x[rs1] == x[rs2] {
                next_pc = cur_pc.wrapping_add(offset as u32);
            }
        }
        Instruction::Bne { rs1, rs2, offset } => {
            if x[rs1] != x[rs2] {
                next_pc = cur_pc.wrapping_add(offset as u32);
            }
        }
        Instruction::Blt { rs1, rs2, offset } => {
            if (x[rs1] as i32) < (x[rs2] as i32) {
                next_pc = cur_pc.wrapping_add(offset as u32);
            }
        }
        Instruction::Bge { rs1, rs2, offset } => {
            if (x[rs1] as i32) >= (x[rs2] as i32) {
                next_pc = cur_pc.wrapping_add(offset as u32);
            }
        }
        Instruction::Bltu { rs1, rs2, offset } => {
            if x[rs1] < x[rs2] {
                next_pc = cur_pc.wrapping_add(offset as u32);
            }
        }
        Instruction::Bgeu { rs1, rs2, offset } => {
            if x[rs1] >= x[rs2] {
                next_pc = cur_pc.wrapping_add(offset as u32);
            }
        }
 
        // ---------- Jumps ----------
        Instruction::Jal { rd, offset } => {
            x[rd] = cur_pc.wrapping_add(4);
            next_pc = cur_pc.wrapping_add(offset as u32);
        }
        Instruction::Jalr { rd, rs1, imm } => {
            // compute the target BEFORE writing rd, because rd and rs1 can be the same register
            let target = x[rs1].wrapping_add(imm as u32) & !1;
            x[rd] = cur_pc.wrapping_add(4);
            next_pc = target;
        }
 
        // ---------- Upper immediates ----------
        Instruction::Lui { rd, imm } => x[rd] = (imm as u32) << 12,
        Instruction::Auipc { rd, imm } => x[rd] = cur_pc.wrapping_add((imm as u32) << 12),
 
        // ---------- System ----------
        Instruction::Ecall => halt = true,
        Instruction::Ebreak => halt = true,
        Instruction::Fence => {} // nothing to do in a simple single-core emulator
 
        Instruction::Illegal(raw) => {
            panic!("illegal instruction {:#010x} at pc {:#x}", raw, cur_pc);
        }
    }
 
    x[0] = 0; // x0 is always zero, even if an instruction wrote to it
    *pc = next_pc;
    halt
}

}
