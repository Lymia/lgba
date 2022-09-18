.section .lgba.header

.arm
.global __start
.align

@
@ The GBA Header
@
__start:
    b ._lgba_init

    @ GBA header, left empty for `gbafix` (for now)
    .space 0xBC

    @ Multiboot header
    0: b 0b @ multiboot entry point; not currently supported
    .space 0x1C
    0: b 0b @ joybus entry point; not currently supported

    @ LGBA specific markers
    .ascii "lgbarom0"
    .word __lgba_exheader

@
@ An extra LGBA-specific header used for (eventually) the ROM builder script.
@
.section .lgba.exheader
__lgba_exheader:
    .word __lgba_exh_lib_cname     @ LGBA crate name
    .word __lgba_exh_lib_cver      @ LGBA crate version
    .word __lgba_exh_rom_cname     @ ROM crate name
    .word __lgba_exh_rom_cver      @ ROM crate version
    .word __lgba_exh_rom_title     @ ROM header title
    .word __lgba_exh_rom_code      @ ROM header code
    .word __lgba_exh_rom_developer @ ROM header developer
    .word __lgba_exh_rom_ver       @ ROM header version

.section .lgba.init
.global __lgba_init_memory

@
@ The entry point for the actual ROM
@
._lgba_init:
    @ Set IRQ stack pointer
    mov r0, #0x12
    msr CPSR_c, r0
    ldr sp, =0x3007FA0

    @ Set user stack pointer
    mov r0, #0x1f
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

    @ Blanks the screen
    ldr r0, =0x4000000
    ldr r1, =0x0080
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
@ Initialize the user memory of lgba
@
__lgba_init_memory:
    push {lr}
    push {r4-r7}

    @ Sets up constants before-hand
    ldr r4, =0x40000D4 @ DMA3SAD
    ldr r5, =0x40000D8 @ DMA3DAD
    ldr r6, =0x40000DC @ DMA3CNT_L
    ldr r7, =0x40000DE @ DMA3CNT_H

    @ Clear .bbs
    ldr r0, =__bss_start
    ldr r1, =__bss_end
    cmp r0, r1
    beq 1f
    ldr r2, =0f
    str r2, [r4]    @ DMA3SAD = &0u32
    ldr r3, =0x8500 @ dma flags = (source fixed, dest increment, 32-bit)
    bl 2f           @ call dma helper function

    @ Set up DMA flags for next section
1:  ldr r3, =0x8400 @ dma flags = (source increment, dest increment, 32-bit)

    @ Copy .iwram section to IWRAM
    ldr r0, =__iwram_start
    ldr r1, =__iwram_end
    cmp r0, r1
    beq 1f
    ldr r2, =__iwram_lma
    str r2, [r4]    @ DMA3SAD = __iwram_lma
    bl 2f           @ call dma helper function

    @ Copy .ewram section to EWRAM
1:  ldr r0, =__ewram_start
    ldr r1, =__ewram_end
    cmp r0, r1
    beq 1f
    ldr r2, =__ewram_lma
    str r2, [r4]    @ DMA3SAD = __ewram_lma
    bl 2f           @ call dma helper function

    @ Return from the function
1:  pop {r4-r7}
    pop {r0}
    bx r0

    @ DMA helper function
2:  sub r1, r0      @ begin computing the word count
    add r1, #3
    lsr r1, #2
    strh r1, [r6]   @ DMA3CNT_L = (end - start + 3) / 4
    str r0, [r5]    @ DMA3DAD = start
    strh r3, [r7]   @ DMA3CNT_H = <begin dma>
    nop             @ \
    nop             @ | wait for the DMA to finish
    nop             @ /
    bx lr

.align 4
0:  .word 0
.pool
