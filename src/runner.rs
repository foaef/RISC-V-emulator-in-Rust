pub fn run(v: &Vec<u8>) {
	// registrii
	let mut PC: u32 = 0;

	while (PC as usize) < v.len() {
		/* pasul 1   fetch
			daca primul octet citit are ultimii doi biti 1 atunci
				instructiunea are 4 octeti
			altfel
				instructiunea are 2 octeti
		*/
		let mut instr: u32 = v[PC as usize].into();
		let lun = if instr & 0b11 == 0b11 { 4 } else { 2 };
		if PC as usize + lun > v.len() {
			panic!("instructiune prea scurta");
		}
		for i in 1..lun {
			instr |= (v[i + PC as usize] as u32) << 8 * i;
		}
		
		/* pasul 2   decode
			https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf
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
			let opcode = I(6,  0);
			let rd     = I(11, 7);
			let funct3 = I(14, 12);
			let rs1    = I(19, 15);
			let rs2    = I(24, 20);
			let funct7 = I(31, 25);
			// wip
		
		} else {
			// wip
		}
	
		/* pasul 3   execute
		 */

		PC += lun as u32;
	}
}
