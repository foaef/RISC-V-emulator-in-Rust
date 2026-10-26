# suma lui gauss
	addi x5, x0, 0     ; sum = 0
	addi x6, x0, 1     ; i = 1
	addi x7, x0, 11    ; limit = 11
label: 
	add  x5, x5, x6    ; sum += i      <- loop starts here
	addi x6, x6, 1     ; i += 1
	bne  x6, x7, label ; if i != limit, jump back to 0x0c
	addi x8, x0, 256   ; x8 = address 256
	sw   x5, 0(x8)     ; mem[256] = sum
	lw   x9, 0(x8)     ; x9 = mem[256]
	ecall              ; stop
