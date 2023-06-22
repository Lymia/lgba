    .section .iwram.lgba_save, "ax", %progbits

@
@ char WramReadByte(const char* offset);
@
@ A routine that reads a byte from a given memory offset.
@
    .thumb_func
    .global __lgba_internal_WramReadByte
__lgba_internal_WramReadByte:
    ldrb r0, [r0]
    bx lr

@
@ bool WramVerifyBuf(const char* buf1, const char* buf2, int count);
@
@ A routine that compares two memory offsets.
@
    .thumb_func
    .global __lgba_internal_WramVerifyBuf
__lgba_internal_WramVerifyBuf:
    push {r4-r5, lr}
    movs r5, r0     @ set up r5 to be r0, so we can use it immediately for the return result
    movs r0, #0     @ set up r0 so the default return result is false

    @ At this point, buf1 is actually in r5, so r0 can be used as a status return
1:  ldrb r3, [r5,r2]
    ldrb r4, [r1,r2]
    cmp r3, r4
    bne 0f
    sub r2, #1
    bpl 1b

    @ Returns from the function successfully
    movs r0, #1
0:  @ Jumps to here return the function unsuccessfully, because r0 contains 0 at this point
    pop {r4-r5}
    pop {r1}
    bx r1

@
@ void WramTransferBuf(const char* source, char* dest, int count);
@
@ A routine that copies one buffer into another.
@
    .thumb_func
    .global __lgba_internal_WramTransferBuf
__lgba_internal_WramTransferBuf:
0:  sub r2, #1
    ldrb r3, [r0,r2]
    strb r3, [r1,r2]
    bne 0b
    bx lr
