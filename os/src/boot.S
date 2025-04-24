.section ".text.boot"

.globl _start
_start:
	/* Mask all interrupts */
	csrw sie, zero

	/* set the stack of SP, size 4KB */
	la sp, stacks_start
	li t0, 40960
	add sp, sp, t0
	/* Clear sscratch */
    csrw sscratch, zero
	/* goto C */
	call kernel_main

.section .data.stack
.align  12
.global stacks_start
stacks_start:
	.skip 40960
.global stacks_end
stacks_end:
