.macro SAVE n
    sd x\n, \n*8(sp)
.endm

.macro LOAD n
    ld x\n, \n*8(sp)
.endm

    .section .text
    .globl _kernel_trap
.align 3
_kernel_trap:
    addi sp, sp, -8*38
    SAVE 1
    SAVE 2
    SAVE 3
    SAVE 4
    SAVE 5
    SAVE 6
    SAVE 7
    SAVE 8
    SAVE 9
    SAVE 10
    SAVE 11
    SAVE 12
    SAVE 13
    SAVE 14
    SAVE 15
    SAVE 16
    SAVE 17
    SAVE 18
    SAVE 19
    SAVE 20
    SAVE 21
    SAVE 22
    SAVE 23
    SAVE 24
    SAVE 25
    SAVE 26
    SAVE 27
    SAVE 28
    SAVE 29
    SAVE 30
    SAVE 31

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
    LOAD 31
    LOAD 30
    LOAD 29
    LOAD 28
    LOAD 27
    LOAD 26
    LOAD 25
    LOAD 24
    LOAD 23
    LOAD 22
    LOAD 21
    LOAD 20
    LOAD 19
    LOAD 18
    LOAD 17
    LOAD 16
    LOAD 15
    LOAD 14
    LOAD 13
    LOAD 12
    LOAD 11
    LOAD 10
    LOAD 9
    LOAD 8
    LOAD 7
    LOAD 6
    LOAD 5
    LOAD 4
    LOAD 3
    LOAD 1

    LOAD 2
    addi sp, sp, 8*39
    sret
