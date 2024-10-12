.altmacro
.macro save_reg n
    sd x\n,\n*8(sp)
.endm
.macro restore_reg n
    ld x\n,\n*8(sp)
.endm
/*
do_exception_vector()必须按4字节对齐
否则写入stvec会不成功
*/
.align 2
.globl do_exception_vectors
do_exception_vectors:
    /*检查sstatus的SPP判断是否异常来自U mode*/
    # x4不需要保存，所以可以用来保存临时值
    csrr x4, sstatus
    andi x4, x4, 1 << 8  # Check SPP bit
    bnez x4, from_s_mode

from_u_mode:
    # sscratch保存的是S模式的sp，将S模式的栈指针给sp，U模式的sp保存到sscratch
    csrrw sp, sscratch, sp
    addi sp,sp,-35*8
    j common_handler

from_s_mode:
    addi sp,sp,-35*8
    sd x2,2*8(sp)
    j common_handler

common_handler:
    sd x1,8(sp)
    sd x3,3*8(sp)
    .set n,5
    .rept 27
        save_reg %n
        .set n,n+1
    .endr

    csrr s1, sstatus
    sd s1, 32*8(sp)

    csrr s2,sepc
    sd s2,0(sp)

    csrr s3,stval
    sd s3,33*8(sp)

    csrr s4,scause
    sd s4,34*8(sp)

    # 不需要保存sscratch就直接跳过
    bnez x4, extern_s_mode

    # 将栈底指针保存到sscratch，以便于下次trap找到sp
    addi s0,sp,35*8
    csrrw s0,sscratch,s0

    # 保存U mode sp指针
    sd s0,2(sp)
    la ra,ret_from_exception

    /*传递两个参数：保存trap上下文的栈sp和scause*/
    mv a0,sp
    mv a1,s4
    tail do_exception

extern_s_mode:
    la ra,ret_from_exception

    /*传递两个参数：保存trap上下文的栈sp和scause*/
    mv a0,sp
    mv a1,s4
    tail do_exception

ret_from_exception:
    ld s1,32*8(sp)
    csrw sstatus,s1

    ld s2,0(sp)
    csrw sepc,s2

    /*保存返回值*/
    sd a0,4*8(sp)

    ld x1,8(sp)
    ld x3,3*8(sp)
    .set n,5
    .rept 27
        restore_reg %n
        .set n,n+1
    .endr

    # 恢复原始栈指针，实际上完成了栈空间的回收
    ld x2,2*8(sp)

    ld a0,4*8(sp)

    sret