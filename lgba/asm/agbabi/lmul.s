@===============================================================================
@
@ ABI:
@    __aeabi_lmul, __aeabi_llsl, __aeabi_llsr, __aeabi_lasr
@
@ Copyright (C) 2021-2023 agbabi contributors
@ For conditions of distribution and use, see copyright notice in LICENSE.md
@
@===============================================================================
@
@ Modified for lgba
@ Copyright (C) 2023 Lymia Kanokawa
@
@ 2023/06/21 - Removed __aeabi_llsl, __aeabi_llsr and __aeabi_lasr in favor of
@              implementations from libaeabi-cortexm0.
@
@===============================================================================

    .arm
    .align 2

    .section .iwram.__aeabi_lmul, "ax", %progbits
    .global __aeabi_lmul
    .type __aeabi_lmul, %function
__aeabi_lmul:
    mul     r3, r0, r3
    mla     r1, r2, r1, r3
    umull   r0, r3, r2, r0
    add     r1, r1, r3
    bx      lr
