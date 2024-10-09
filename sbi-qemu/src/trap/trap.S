/*
    _sbi_exception_vector
    M模式下的异常向量表入口
    按8字节对齐
*/
/*必须加上.altmacro，不然就用不了宏*/
.altmacro

.macro save_reg n
    sd x\n, \n*8(sp)
.endm

.macro restore_reg n
    ld x\n, \n*8(sp)
.endm

.align 3
.globl _sbi_exception_vector
_sbi_exception_vector:
    /*从mscratch 获取M模式的sp指针，并把S模式的sp保存到mscratch*/
    csrrw sp,mscratch,sp
    addi sp,sp,-33*8
    sd x1,8(sp)
    sd x3,3*8(sp)
    .set n, 5
    .rept 27
        save_reg %n
        .set n,n+1
    .endr

    csrr t0,mepc
    sd t0,0(sp)

    csrr t0,mstatus
    sd t0,32*8(sp)

    /*将M模式下栈底指针保存到mscratch，以便于下次陷入M模式时可以找到sp*/
    addi t0,sp,33*8
    csrrw t0,mscratch,t0
    sd t0,2*8(sp)

    mv a0,sp
    call sbi_trap_handler

    /*保存返回值*/
    sd a0,4*8(sp)

    ld t0,32*8(sp)
    csrw mstatus,t0

    ld t0,0(sp)
    csrw mepc,t0

    ld x1,8(sp)
    ld x3,3*8(sp)
    .set n,5
    .rept 27
        restore_reg %n
        .set n,n+1
    .endr

    /*恢复返回值*/
    ld a0,4*8(sp)

    ld sp,2*8(sp)

    mret

