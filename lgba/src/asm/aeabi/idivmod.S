/* Runtime ABI for the ARM Cortex-M0  
 * idivmod.S: signed 32 bit division (quotient and remainder)
 *
 * Copyright (c) 2012 Jörg Mische <bobbl@gmx.de>
 *
 * Permission to use, copy, modify, and/or distribute this software for any
 * purpose with or without fee is hereby granted, provided that the above
 * copyright notice and this permission notice appear in all copies.
 *
 * THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
 * WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
 * MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
 * ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
 * WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN
 * ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
 * OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 */



	.syntax unified
	.text
	.thumb
	.cpu arm7tdmi



@ {int quotient:r0, int remainder:r1}
@ __aeabi_idivmod(int numerator:r0, int denominator:r1)
@
@ Divide r0 by r1 and return the quotient in r0 and the remainder in r1
@
	.thumb_func
        .global __aeabi_idivmod
__aeabi_idivmod:

	cmp	r0, #0
	bge	.Lidm_numerator_pos
	rsbs	r0, r0, #0		@ num = -num
	cmp	r1, #0
	bge	.Lidm_both_neg

	rsbs	r1, r1, #0		@ den = -den
	push	{lr}
	bl	__aeabi_uidivmod
	rsbs	r1, r1, #0		@ rem = -rem
	pop	{pc}

.Lidm_both_neg:
	push	{lr}
	bl	__aeabi_uidivmod
	rsbs	r0, r0, #0		@ quot = -quot
	rsbs	r1, r1, #0		@ rem = -rem
	pop	{pc}

.Lidm_numerator_pos:
	cmp	r1, #0
	bge	.Lidm_uidivmod

	rsbs	r1, r1, #0		@ den = -den
	push	{lr}
	bl	__aeabi_uidivmod
	rsbs	r0, r0, #0		@ quot = -quot
	pop	{pc}





@ unsigned __udivsi3(unsigned num, unsigned denom)
@
@ libgcc wrapper: just an alias for __aeabi_uidivmod(), the remainder is ignored
@
	.thumb_func
        .global __udivsi3
__udivsi3:



@ unsigned __aeabi_uidiv(unsigned num, unsigned denom)
@
@ Just an alias for __aeabi_uidivmod(), the remainder is ignored
@
	.thumb_func
        .global __aeabi_uidiv
__aeabi_uidiv:



@ {unsigned quotient:r0, unsigned remainder:r1}
@  __aeabi_uidivmod(unsigned numerator:r0, unsigned denominator:r1)
@
@ Divide r0 by r1 and return the quotient in r0 and the remainder in r1
@
	.thumb_func
        .global __aeabi_uidivmod
__aeabi_uidivmod:



.Lidm_uidivmod:
	cmp	r1, #0
	bne	1f

	@ call __aeabi_idiv0
	ldr r0, =__aeabi_idiv0
	bx  r0
	.pool
1:

	@ Shift left the denominator until it is greater than the numerator
	movs	r2, #1		@ counter
	movs	r3, #0		@ result
	cmp	r0, r1
	bls	.Lidm_sub_loop
	adds	r1, #0		@ dont shift if denominator would overflow
	bmi	.Lidm_sub_loop

.Lidm_denom_shift_loop:
	lsls	r2, #1
	lsls	r1, #1
	bmi	.Lidm_sub_loop
	cmp	r0, r1
	bhi	.Lidm_denom_shift_loop

.Lidm_sub_loop:	
	cmp	r0, r1
	bcc	.Lidm_dont_sub	@ if (num>denom)

	subs	r0, r1		@ numerator -= denom
	orrs	r3, r2		@ result(r3) |= bitmask(r2)
.Lidm_dont_sub:

	lsrs	r1, #1		@ denom(r1) >>= 1
	lsrs	r2, #1		@ bitmask(r2) >>= 1
	bne	.Lidm_sub_loop

	movs	r1, r0		@ remainder(r1) = numerator(r0)
	movs	r0, r3		@ quotient(r0) = result(r3)
	bx	lr
