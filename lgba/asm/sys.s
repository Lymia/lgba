@
@ A dummy implementation of this to deal with a LLVM bug.
@ Put here even partly just to zero the temptation to copy this into code...
@
    .section .text, "ax", %progbits
    .thumb
    .global __sync_synchronize
__sync_synchronize:
    bx lr

@
@ Abort function for shutting down the GBA in case of an error.
@
@ The name of this function *IS* stable API, in case you're writing something that needs to call this manually.
@
    .section .text, "ax", %progbits
    .thumb
    .global __lgba_abort
__lgba_abort:
    mov r0, #0           @ r0 = 0
    mov r1, #1           @ r1 = 1

    lsl r2, r1, #26      @ r2 = 0x4000000
    lsl r3, r1, #9       @ r3 = 0x200
    add r4, r2, r3       @ r4 = 0x4000200

    @ Disable interrupts
    strh r0, [r4, #0x08] @ IME (0x4000208)
    strh r0, [r4, #0x00] @ IE  (0x4000200)
    strh r0, [r4, #0x02] @ IF  (0x4000202)

    @ Disable DMA
    add r2, #0xB0        @ r2 = 0x40000B0
    strh r0, [r2, #0x0A] @ DMA0CNT_H (0x40000BA)
    strh r0, [r2, #0x16] @ DMA0CNT_H (0x40000C6)
    strh r0, [r2, #0x22] @ DMA0CNT_H (0x40000D2)
    strh r0, [r2, #0x2E] @ DMA0CNT_H (0x40000DE)

    @ Disable sound (so we don't blast the player with crunch noises)
    sub r2, #0x30        @ r2 = 0x4000080
    strh r0, [r2]        @ SOUNDCNT_L (0x4000080)

    @ Jump to the wait loop in EWRAM
    ldr r2, 0f           @ r2 = (wait loop)
    lsl r3, r1, #25      @ r3 = 0x2000000
    str r2, [r3]
    add r3, #1           @ r3 = 0x2000001
    bx r3

    @ The wait loop itself.
    .align 4
0:  swi #0x02
    b 0b
.pool
