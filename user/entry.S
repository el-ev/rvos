	.section .text
	.globl _start

_start:
	mv a0, sp
	ori a1, x0, 0x1
	ld  a2, 0(a1)
	j main

