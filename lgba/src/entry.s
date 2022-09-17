.section .lgba.header

.arm
.global __start
.align

@
@ The GBA Header
@
__start:
    b ._lgba_init

    @ Left empty for `gbafix`
    .space 188

@
@ The entry point for the actual ROM
@
._lgba_init:
    @ set IRQ stack pointer
    mov r0, #0x12
    msr CPSR_c, r0
    ldr sp, =0x03007FA0

    @ set user stack pointer
    mov r0, #0x1f
    msr CPSR_c, r0
    ldr sp, =0x03007F00

    @ Sets WAITCNT to the default used by GBA games
    @
    @ See https://problemkaputt.de/gbatek.htm#gbasystemcontrol for reference.
    ldr r0, =0x04000204
    ldr r1, =0x4317
    strh r1, [r0]

    @ Initializes memory
    bl ._lgba_init_memory

    @ Call lgba initialization code
    ldr r0, =__lgba_init_rust
    bl 2f

    @ jump to user code
    ldr r0, =main
    bl 2f

    @ main should be `fn() -> !`, but it doesn't hurt to guard
    1: b 1b

    @ trampoline for blx
    2: bx r0

@
@ Initialize the user memory of lgba
@
._lgba_init_memory:
    @ function prologue
    push {lr}
    push {r4-r10}

    @ zeros .bbs
1:  ldr r0, =__bss_start
    ldr r1, =__bss_end
    cmp r0, r1
    beq 1f
    mov r3, #0
    mov r4, #0
    mov r5, #0
    mov r6, #0
    mov r7, #0
    mov r8, #0
    mov r9, #0
    mov r10, #0
0:  stmia r0!,{r3-r10}
    cmp r0, r1
    blt 0b

    @ copy .iwram section to IWRAM
1:  ldr r0, =__iwram_lma
    ldr r1, =__iwram_end
    ldr r2, =__iwram_start
    cmp r1, r2
    beq 1f
0:  ldmia r0!,{r3-r10}
    stmia r2!,{r3-r10}
    cmp r2, r1
    blt 0b

    @ copy .ewram section to EWRAM
1:  ldr r0, =__ewram_lma
    ldr r1, =__ewram_end
    ldr r2, =__ewram_start
    cmp r1, r2
    beq 1f
0:  ldmia r0!,{r3-r10}
    stmia r2!,{r3-r10}
    cmp r2, r1
    blt 0b

    @ function epilogue
1:  pop {r4-r10}
    pop {r0}
    bx r0
