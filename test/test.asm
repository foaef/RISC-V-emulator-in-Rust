addi x5, x0, 0
addi x6, x0, 1
addi x7, x0, 11
label:
add  x5, x5, x6
addi x6, x6, 1
bne  x6, x7, label
addi x8, x0, 256
sw   x5, 0(x8)
lw   x9, 0(x8)
ecall
