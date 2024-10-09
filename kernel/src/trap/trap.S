.macro SAVEK n
    sd x\n, \n*8(sp)
.endm

.macro LOADK n
    ld x\n, \n*8(sp)
.endm

.macro SAVE reg, off
    sd \reg, \off*8(a0)
.endm

.macro LOAD reg, off
    ld \reg, \off*8(a0)
.endm

    .section .text
    .globl _kernel_to_kernel_trap
    .type _kernel_to_kernel_trap, @function
.align 3
_kernel_to_kernel_trap:
    addi sp, sp, -8*38
    SAVEK 1
    SAVEK 2
    SAVEK 3
    SAVEK 4
    SAVEK 5
    SAVEK 6
    SAVEK 7
    SAVEK 8
    SAVEK 9
    SAVEK 10
    SAVEK 11
    SAVEK 12
    SAVEK 13
    SAVEK 14
    SAVEK 15
    SAVEK 16
    SAVEK 17
    SAVEK 18
    SAVEK 19
    SAVEK 20
    SAVEK 21
    SAVEK 22
    SAVEK 23
    SAVEK 24
    SAVEK 25
    SAVEK 26
    SAVEK 27
    SAVEK 28
    SAVEK 29
    SAVEK 30
    SAVEK 31

    csrr t0, sstatus
    sd t0, 8*32(sp)
    csrr t0, sepc
    sd t0, 8*33(sp)
    csrr t0, satp
    sd t0, 8*34(sp)
    sd sp, 8*35(sp)
    csrr t0, stval
    sd t0, 8*36(sp)
    sd tp, 8*37(sp)

    mv a0, sp
    csrr a1, scause
    csrr a2, stval
    call kernel_trap_handler

    ld t0, 8*36(sp)
    csrw stval, t0
    ld t0, 8*34(sp)
    csrw satp, t0
    ld t0, 8*33(sp)
    csrw sepc, t0
    ld t0, 8*32(sp)
    csrw sstatus, t0
    LOADK 31
    LOADK 30
    LOADK 29
    LOADK 28
    LOADK 27
    LOADK 26
    LOADK 25
    LOADK 24
    LOADK 23
    LOADK 22
    LOADK 21
    LOADK 20
    LOADK 19
    LOADK 18
    LOADK 17
    LOADK 16
    LOADK 15
    LOADK 14
    LOADK 13
    LOADK 12
    LOADK 11
    LOADK 10
    LOADK 9
    LOADK 8
    LOADK 7
    LOADK 6
    LOADK 5
    LOADK 4
    LOADK 3
    LOADK 1

    LOADK 2
    addi sp, sp, 8*38
    sret
    .size _kernel_to_kernel_trap, . - _kernel_to_kernel_trap


    .globl _kernel_to_user_trap
    .type _kernel_to_user_trap, @function
_kernel_to_user_trap:
    SAVE s0, 34
    SAVE s1, 35
    SAVE s2, 36
    SAVE s3, 37
    SAVE s4, 38
    SAVE s5, 39
    SAVE s6, 40
    SAVE s7, 41
    SAVE s8, 42
    SAVE s9, 43
    SAVE s10, 44
    SAVE s11, 45
    SAVE ra, 46
    SAVE sp, 47
    SAVE tp, 48

    csrw sscratch, a0

    LOAD t0, 32
    csrw sepc, t0
    LOAD t0, 33
    csrw scause, t0

    LOAD x1, 1
    LOAD x2, 2
    LOAD x3, 3
    LOAD x4, 4
    LOAD x5, 5
    LOAD x6, 6
    LOAD x7, 7
    LOAD x8, 8
    LOAD x9, 9
    //
    LOAD x11, 11
    LOAD x12, 12
    LOAD x13, 13
    LOAD x14, 14
    LOAD x15, 15
    LOAD x16, 16
    LOAD x17, 17
    LOAD x18, 18
    LOAD x19, 19
    LOAD x20, 20
    LOAD x21, 21
    LOAD x22, 22
    LOAD x23, 23
    LOAD x24, 24
    LOAD x25, 25
    LOAD x26, 26
    LOAD x27, 27
    LOAD x28, 28
    LOAD x29, 29
    LOAD x30, 30
    LOAD x31, 31

    LOAD x10, 10
    sret
    .size _kernel_to_user_trap, . - _kernel_to_user_trap

    .globl _user_to_kernel_trap
    .type _user_to_kernel_trap, @function
_user_to_kernel_trap:
    csrrw a0, sscratch, a0

    SAVE x1, 1
    SAVE x2, 2
    SAVE x3, 3
    SAVE x4, 4
    SAVE x5, 5
    SAVE x6, 6
    SAVE x7, 7
    SAVE x8, 8
    csrr x9, sscratch
    SAVE x9, 10
    SAVE x11, 11
    SAVE x12, 12
    SAVE x13, 13
    SAVE x14, 14
    SAVE x15, 15
    SAVE x16, 16
    SAVE x17, 17
    SAVE x18, 18
    SAVE x19, 19
    SAVE x20, 20
    SAVE x21, 21
    SAVE x22, 22
    SAVE x23, 23
    SAVE x24, 24
    SAVE x25, 25
    SAVE x26, 26
    SAVE x27, 27
    SAVE x28, 28
    SAVE x29, 29
    SAVE x30, 30
    SAVE x31, 31

    csrr t0, sepc
    SAVE t0, 32
    csrr t0, scause
    SAVE t0, 33

    LOAD s0, 34
    LOAD s1, 35
    LOAD s2, 36
    LOAD s3, 37
    LOAD s4, 38
    LOAD s5, 39
    LOAD s6, 40
    LOAD s7, 41
    LOAD s8, 42
    LOAD s9, 43
    LOAD s10, 44
    LOAD s11, 45
    LOAD ra, 46
    LOAD sp, 47
    LOAD tp, 48

    ret
    .size _user_to_kernel_trap, . - _user_to_kernel_trap