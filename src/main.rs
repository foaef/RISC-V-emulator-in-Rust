mod cpu;
use cpu::Instruction;
 
fn main() {
    // A tiny hand-assembled RV32I program: sum = 1 + 2 + ... + 10
    // (the comment on each line is the assembly it was written from)
    let program: [u32; 10] = [
        0x00000293, // 0x00: addi x5, x0, 0      sum = 0
        0x00100313, // 0x04: addi x6, x0, 1      i = 1
        0x00B00393, // 0x08: addi x7, x0, 11     limit = 11
        0x006282B3, // 0x0c: add  x5, x5, x6     sum += i      <- loop starts here
        0x00130313, // 0x10: addi x6, x6, 1      i += 1
        0xFE731CE3, // 0x14: bne  x6, x7, -8     if i != limit, jump back to 0x0c
        0x10000413, // 0x18: addi x8, x0, 256    x8 = address 256
        0x00542023, // 0x1c: sw   x5, 0(x8)      mem[256] = sum
        0x00042483, // 0x20: lw   x9, 0(x8)      x9 = mem[256]
        0x00000073, // 0x24: ecall               stop
    ];
 
    // 1. "Boot" the machine: 1 KiB of memory, registers all zero, pc = 0
    let mut mem = vec![0u8; 1024];
    let mut x = [0u32; 32];
    let mut pc: u32 = 0;
 
    // 2. Copy the program into memory (RISC-V is little-endian)
    for (i, word) in program.iter().enumerate() {
        mem[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
 
    // 3. The fetch -> decode -> execute loop
    loop {
        let mut len = 0;
        let raw = Instruction::fetch_instruction(&mem, &mut pc, &mut len);
        let instr = Instruction::decode_instruction(raw);
        println!("pc={:#06x}  raw={:#010x}  {:?}", pc, raw, instr);
 
        let halt = Instruction::execute_instruction(&mut x, &mut mem, &mut pc, instr);
        if halt {
            break;
        }
    }
 
    // 4. Show the result
    println!();
    println!("x5 (sum)           = {}", x[5]);
    println!("x9 (loaded back)   = {}", x[9]);
    println!("mem[256..260]      = {:?}", &mem[256..260]);
}
