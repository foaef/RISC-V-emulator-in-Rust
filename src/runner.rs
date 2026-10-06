pub fn run(v: &Vec<u8>) {
	// registrii
	let mut pc: u32 = 0;
	let mut x: [u32; 32] = [0; 32];
	let mut f: [u32; 32] = [0; 32];

	while (pc as usize) < v.len() {
		x[0] = 0;

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
		let bad = || { panic!("instructiune gresita sau neimplementata"); };
		if lun == 4 {
			// I(n)(l,r) <=> n[l:r]
			let I = |n: u32| { move |l: usize, r: usize| {
				let (l, r) = (if l > r { l } else { r }, if l > r { r } else { l });
				(n >> r) & ((1 << l - r + 1) - 1)
			}};
			// E(n)(c) <=> n extins cu msb-ul de pe pozitia c
			let E = |n: u32| { move |c: u32| {
				(((n as i32) << 32 - c - 1) >> 32 - c - 1) as u32
			}};

			let opcode = I(instr)(6, 0);
			match opcode {
				/* (R)    */ 0b0110011 => {
					let rd     = I(instr)(11, 7) as usize;
					let funct3 = I(instr)(14, 12);
					let rs1    = I(instr)(19, 15) as usize;
					let rs2    = I(instr)(24, 20) as usize;
					let funct7 = I(instr)(31, 25);
					match (funct3, funct7) {
						/* add  */ (0x0, 0x00) => x[rd] = x[rs1] + x[rs2],
						/* sub  */ (0x0, 0x20) => x[rd] = x[rs1] - x[rs2],
						/* xor  */ (0x4, 0x00) => x[rd] = x[rs1] ^ x[rs2],
						/* or   */ (0x6, 0x00) => x[rd] = x[rs1] | x[rs2],
						/* and  */ (0x7, 0x00) => x[rd] = x[rs1] & x[rs2],
						/* sll  */ (0x1, 0x00) => x[rd] = x[rs1] << x[rs2],
						/* srl  */ (0x5, 0x00) => x[rd] = x[rs1] >> x[rs2],
						/* sra  */ (0x5, 0x20) => x[rd] = ((x[rs1] as i32) >> x[rs2]) as u32,
						/* slt  */ (0x2, 0x00) => x[rd] = if (x[rs1] as i32) < (x[rs2] as i32) { 1 } else { 0 },
						/* sltu */ (0x3, 0x00) => x[rd] = if x[rs1] < x[rs2] { 1 } else { 0 },
						/*      */ (___, ____) => bad()
					}
				},
				/* (I #1) */ 0b0010011 => {
					let rd     = I(instr)(11, 7) as usize;
					let funct3 = I(instr)(14, 12);
					let rs1    = I(instr)(19, 15) as usize;
					let imm    = E(I(instr)(31, 20))(11);
					match (funct3, I(imm)(11, 5)) {
						/* slli  */ (0x1, 0x00) => x[rd] = x[rs1] << I(imm)(0, 4),
						/* srli  */ (0x5, 0x00) => x[rd] = x[rs1] >> I(imm)(0, 4),
						
						/* srai  */ (0x5, 0x20) => x[rd] = ((x[rs1] as i32) >> I(imm)(0, 4)) as u32,
						
						/* addi  */ (0x0, ____) => x[rd] = x[rs1] + imm,
						/* xori  */ (0x4, ____) => x[rd] = x[rs1] ^ imm,
						/* ori   */ (0x6, ____) => x[rd] = x[rs1] | imm,
						/* andi  */ (0x7, ____) => x[rd] = x[rs1] & imm,
						/* slti  */ (0x2, ____) => x[rd] = if (x[rs1] as i32) < (imm as i32) { 1 } else { 0 },
						/* sltiu */ (0x3, ____) => x[rd] = if x[rs1] < imm { 1 } else { 0 },
						/*       */ (___, ____) => bad()
					}
				},
				/* (I #2) */ 0b0000011 => {
					let rd     = I(instr)(11, 7) as usize;
					let funct3 = I(instr)(14, 12);
					let rs1    = I(instr)(19, 15) as usize;
					let imm    = E(I(instr)(31, 20))(11);
					// wip
				},
				// wip
				/*       */  _________ => bad()
			}
		} else {
			// wip
		}
	
		pc += lun as u32;
	}
}
