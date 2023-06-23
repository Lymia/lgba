@
@ The entry point for the actual ROM
@
    .arm
    .global __lgba__internal_start
__lgba__internal_start:
    @ Set IRQ stack pointer
    movs r0, #0x12
    msr CPSR_c, r0
    ldr sp, =0x3007FA0

    @ Set user stack pointer
    movs r0, #0x1f
    msr CPSR_c, r0
    ldr sp, =0x3007F00

    @ Switch to Thumb
    ldr r0, =(1f + 1)
    bx r0
    .thumb
    .align 2
    1:

    @ Sets WAITCNT to the default used by GBA games
    ldr r0, =0x4000204
    ldr r1, =0x4317
    strh r1, [r0]

    @ Initializes memory
    bl __lgba_init_memory

    @ Call lgba initialization code
    ldr r0, =__lgba_init_rust
    bl 2f

    @ Jump to user code
    ldr r0, =__lgba_rom_entry
    bl 2f

    @ Call a fallback function that just panics.
    ldr r0, =__lgba_main_func_returned
    bl 2f

    @ This should be *completely* unreachable, but... just in case.
1:  b 1b

    @ Trampoline for blx - we don't know if these functions are ARM or Thumb (since we support armv4t target)
2:  bx r0
.pool

@
@ The entry point for the ROM in multiplay transfer environments
@
    .arm
    .global __lgba__internal_multiplay_start
__lgba__internal_multiplay_start:
    b __lgba__internal_multiplay_start
.pool

@
@ The entry point for the ROM in joybus environments
@
    .arm
    .global __lgba__internal_joybus_start
__lgba__internal_joybus_start:
    b __lgba__internal_joybus_start
.pool

@
@ Initialize the user memory of lgba
@
@ The name of this function *IS* stable API, in case you're writing something that needs to call this manually.
@
    .thumb_func
    .global __lgba_init_memory
__lgba_init_memory:
    push {r4,lr}

    @ Sets up constants before-hand
    movs r4, #1
    lsls r4, #26
    adds r4, #0xD0         @ r4 = 0x40000D0

    @ Create a value on the stack set to 0x00.
    sub sp, #4
    movs r2, #0
    str r2, [sp]           @ *sp = 0u32
    mov r2, sp             @ r2 = &0u32
    add sp, #4

    @ Clear .bbs
    ldr r0, =__bss_start   @ start of bss
    ldr r1, =__bss_end     @ end of bss
    @ (r2 from earlier)    @ copy source = &0u32
    movs r3, #0x85
    lsls r3, #8            @ dma flags (0x8500)
    bl 1f

    @ Copy .iwram section to IWRAM
    ldr r0, =__iwram_start @ start of iwram
    ldr r1, =__iwram_end   @ end of iwram
    ldr r2, =__iwram_lma   @ iwram data in ROM
    movs r3, #0x84
    lsls r3, #8            @ dma flags (0x8400)
    bl 1f

    @ Copy .ewram section to EWRAM
    ldr r0, =__ewram_start @ start of ewram
    ldr r1, =__ewram_end   @ end of ewram
    ldr r2, =__ewram_lma   @ ewram data in ROM
    @ (r3 carried over)    @ dma flags (0x8400)
    bl 1f

    @ Return from the function
    pop {r4}
    pop {r0}
    bx r0

    @ DMA helper function
1:  cmp r0, r1
    beq 0f                 @ Bail out early if there is nothing to copy
    subs r1, r0
    adds r1, #3
    lsrs r1, #2            @ r1 = (r1 - r0 + 3) / 4
    str  r2, [r4, #0x4]    @ DMA3SAD   (0x40000D4) = r2
    str  r0, [r4, #0x8]    @ DMA3DAD   (0x40000D8) = r0
    strh r1, [r4, #0xC]    @ DMA3CNT_L (0x40000DC) = r1
    strh r3, [r4, #0xE]    @ DMA3CNT_H (0x40000DE) = r3
    nop
    nop
0:  bx lr
.pool

