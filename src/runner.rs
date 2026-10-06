pub fn run(v: &Vec<u8>) {
	// registrii
	let mut pc: u32 = 0;
	let mut x: [u32; 32] = [0; 32];
	let mut f: [u32; 32] = [0; 32];

	while (pc as usize) < v.len() {
		/* pasul 1   fetch
			daca primul octet citit are ultimii doi biti 1 atunci
				instructiunea are 4 octeti
			altfel
				instructiunea are 2 octeti
		*/
		let mut instr: u32 = v[pc as usize].into();
		let lun = if instr & 0b11 == 0b11 { 4 } else { 2 };
		if pc as usize + lun > v.len() {
			panic!("instructiune prea scurta");
		}
		for i in 1..lun {
			instr |= (v[i + pc as usize] as u32) << 8 * i;
		}
		
		/* pasul 2+3   decode+execute
			https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
			de aici sunt si titlurile tabelului
		*/
		if lun == 4 {
			// pt interval [l:r]
			let I = |mut l: usize, mut r: usize| {
				if l < r {
					let t = l;
					l = r;
					r = t
				}
				(instr >> r) & ((1 << (l - r + 1)) - 1)
			};
			
			let opcode        = I(6,  0);
			let rd:  *mut u32 = &mut x[I(11, 7) as usize];
			let funct3        = I(14, 12);
			let rs1: *mut u32 = &mut x[I(19, 15) as usize];
			let rs2: *mut u32 = &mut x[I(24, 20) as usize];
			let funct7        = I(31, 25);

			let mut ok: bool = false;
			// pt actiune
			let mut act = |Opcode: u32, Funct3: u32, Funct7: u32, Description: &dyn Fn()| {
				if Opcode == opcode && Funct3 == funct3 && Funct7 == funct7 {
					ok = true;
					Description();
				}
			};
			unsafe {
				// "RV32I Base Integer Instructions"
				// Name     Opcode        FMT      funct3  funct7  Description (C)                               Note
				/* add  */  act(0b110011, /* R */  0x0,    0x00,   &||{*rd = *rs1 + *rs2});
				/* sub  */  act(0b110011, /* R */  0x0,    0x20,   &||{*rd = *rs1 - *rs2});
				/* xor  */  act(0b110011, /* R */  0x4,    0x00,   &||{*rd = *rs1 ^ *rs2});
				/* or   */  act(0b110011, /* R */  0x6,    0x00,   &||{*rd = *rs1 | *rs2});
				/* and  */  act(0b110011, /* R */  0x7,    0x00,   &||{*rd = *rs1 & *rs2});
				/* sll  */  act(0b110011, /* R */  0x1,    0x00,   &||{*rd = *rs1 << *rs2});
				/* srl  */  act(0b110011, /* R */  0x5,    0x00,   &||{*rd = *rs1 >> *rs2});
				/* sra  */  act(0b110011, /* R */  0x5,    0x20,   &||{*rd = ((*rs1 as i32) >> *rs2) as u32});   /* msb-extends */
				/* slt  */  act(0b110011, /* R */  0x2,    0x00,   &||{*rd = ((*rs1 as i32) < (*rs2 as i32))});
				/* sltu */  act(0b110011, /* R */  0x3,    0x00,   &||{*rd = *rs1 < *rs2});
				// wip

				// "RV32M Multiply Extension"
				// wip

				// wip restul
			}
			if !ok {
				panic!("instructiune invalida/neimplementata");
			}
		} else {
			// wip
		}
	
		pc += lun as u32;
	}
}
