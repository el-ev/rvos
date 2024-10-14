.macro SAVEK reg, off
    sd \reg, \off*8(sp)
.endm

.macro LOADK reg, off
    ld \reg, \off*8(sp)
.endm

.macro SAVE reg, off
    sd \reg, \off*8(a0)
.endm

.macro LOAD reg, off
    ld \reg, \off*8(a0)
.endm

    .globl _kernel_to_kernel_trap
    .type _kernel_to_kernel_trap, @function
.align 3
_kernel_to_kernel_trap:
    addi sp, sp, -8*17
    SAVEK ra, 0
    SAVEK t0, 1
    SAVEK t1, 2
    SAVEK t2, 3
    SAVEK a0, 4
    SAVEK a1, 5
    SAVEK a2, 6
    SAVEK a3, 7
    SAVEK a4, 8
    SAVEK a5, 9
    SAVEK a6, 10
    SAVEK a7, 11
    SAVEK t3, 12
    SAVEK t4, 13
    SAVEK t5, 14
    SAVEK t6, 15
    csrr t0, sepc
    SAVEK t0, 16
    mv a0, sp
    csrr a1, scause
    csrr a2, stval
    call kernel_trap_handler
    LOADK t0, 16
    csrw sepc, t0
    LOADK t6, 15
    LOADK t5, 14
    LOADK t4, 13
    LOADK t3, 12
    LOADK a7, 11
    LOADK a6, 10
    LOADK a5, 9
    LOADK a4, 8
    LOADK a3, 7
    LOADK a2, 6
    LOADK a1, 5
    LOADK a0, 4
    LOADK t2, 3
    LOADK t1, 2
    LOADK t0, 1
    LOADK ra, 0
    addi sp, sp, 8*17
    sret
    .size _kernel_to_kernel_trap, . - _kernel_to_kernel_trap

    .globl _kernel_to_user_trap
    .type _kernel_to_user_trap, @function
    .align 3

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
    LOAD x10, 10 // x10 is a0
    sret
    .size _kernel_to_user_trap, . - _kernel_to_user_trap

    .globl _user_to_kernel_trap
    .type _user_to_kernel_trap, @function
    .align 3

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