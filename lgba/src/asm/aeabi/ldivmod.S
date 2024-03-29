/* Runtime ABI for the ARM Cortex-M0  
 * ldivmod.S: 64 bit division (quotient and remainder)
 *
 * Copyright (c) 2012-2017 Jörg Mische <bobbl@gmx.de>
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



@ {long long quotient, long long remainder}
@ __aeabi_ldivmod(long long numerator, long long denominator)
@
@ Divide r1:r0 by r3:r2 and return the quotient in r1:r0 and the remainder in
@ r3:r2 (all signed)
@
	.thumb_func
        .global __aeabi_ldivmod
__aeabi_ldivmod:

	cmp	r1, #0
	bge	.Lldm_numerator_pos

	push	{r4, lr}
	movs	r4, #0			@ num = -num
	rsbs	r0, r0, #0
	sbcs	r4, r1
	movs	r1, r4

	cmp	r3, #0
	bge	.Lldm_both_neg

	movs	r4, #0			@ den = -den
	rsbs	r2, r2, #0
	sbcs	r4, r3
	movs	r3, r4
	bl	__aeabi_uldivmod
	movs	r4, #0			@ rem = -rem
	rsbs	r2, r2, #0
	sbcs	r4, r3
	movs	r3, r4
	pop	{r4, pc}
	
.Lldm_both_neg:
	bl	__aeabi_uldivmod
	movs	r4, #0			@ quot = -quot
	rsbs	r0, r0, #0
	sbcs	r4, r1
	movs	r1, r4
	movs	r4, #0			@ rem = -rem
	rsbs	r2, r2, #0
	sbcs	r4, r3
	movs	r3, r4
	pop	{r4, pc}

.Lldm_numerator_pos:
	cmp	r3, #0
	bge	.Lldm_uldivmod

	push	{r4, lr}
	movs	r4, #0			@ den = -den
	rsbs	r2, r2, #0
	sbcs	r4, r3
	movs	r3, r4
	bl	__aeabi_uldivmod
	movs	r4, #0			@ quot = -quot
	rsbs	r0, r0, #0
	sbcs	r4, r1
	movs	r1, r4
	pop	{r4, pc}




@ unsigned long long __udivdi3(unsigned long long num, unsigned long long denom)
@
@ libgcc wrapper: just an alias for __aeabi_uldivmod(), the remainder is ignored
@
	.thumb_func
        .global __udivdi3
__udivdi3:



@ {unsigned long long quotient, unsigned long long remainder}
@ __aeabi_uldivmod(unsigned long long numerator, unsigned long long denominator)
@
@ Divide r1:r0 by r3:r2 and return the quotient in r1:r0 and the remainder
@ in r3:r2 (all unsigned)
@
	.thumb_func
        .global __aeabi_uldivmod
__aeabi_uldivmod:



.Lldm_uldivmod:
	cmp	r3, #0
	bne	.Lldm__large_denom
	cmp	r2, #0
	beq	.Lldm__divison_by_0
	cmp	r1, #0
	beq	.Lldm__fallback_32bits



	@ case 1: num >= 2^32 and denom < 2^32
	@ Result might be > 2^32, therefore we first calculate the upper 32
	@ bits of the result. It is done similar to the calculation of the
	@ lower 32 bits, but with a denominator that is shifted by 32.
	@ Hence the lower 32 bits of the denominator are always 0 and the
	@ costly 64 bit shift and sub operations can be replaced by cheap 32
	@ bit operations.

	push	{r4, r5, r6, r7, lr}

	@ shift left the denominator until it is greater than the numerator
	@ denom(r7:r6) = r3:r2 << 32

	movs	r5, #1		@ bitmask
	adds	r7, r2, #0	@ dont shift if denominator would overflow
	bmi	.Lldm__upper_result
	cmp	r1, r7
	blo	.Lldm__upper_result

.Lldm__denom_shift_loop1:
	lsls	r5, #1
	lsls	r7, #1
	bmi	.Lldm__upper_result	@ dont shift if overflow
	cmp	r1, r7
	bhs	.Lldm__denom_shift_loop1

.Lldm__upper_result:
	movs	r3, r1
	movs	r2, r0
	movs	r1, #0		@ upper result = 0
	b	.Lldm__sub_entry1

.Lldm__sub_loop1:
	lsrs	r7, #1		@ denom(r7:r6) >>= 1

.Lldm__sub_entry1:
	cmp	r3, r7
	bcc	.Lldm__dont_sub1	@ if (num>denom)

	subs	r3, r7		@ num -= denom
	orrs	r1, r5		@ result(r7:r6) |= bitmask(r5)
.Lldm__dont_sub1:

	lsrs	r5, #1		@ bitmask(r5) >>= 1
	bne	.Lldm__sub_loop1

	movs	r5, #1
	lsls	r5, #31
	lsls	r6, r7, #31	@ denom(r7:r6) = (r7:0) >> 1
	lsrs	r7, #1		@ dont forget least significant bit!
	b	.Lldm__lower_result



	@ case 2: division by 0
	@ call __aeabi_ldiv0

.Lldm__divison_by_0:
    ldr r0, =__aeabi_ldiv0
	bx  r0
	.pool



	@ case 3: num < 2^32 and denom < 2^32
	@ fallback to 32 bit division

.Lldm__fallback_32bits:
	movs	r1, r2
	push	{lr}
	bl	__aeabi_uidivmod
	movs	r2, r1
	movs	r1, #0
	movs	r3, #0
	pop	{pc}



	@ case 4: denom >= 2^32
	@ result is smaller than 2^32

.Lldm__large_denom:
	push	{r4, r5, r6, r7, lr}

	movs	r7, r3
	movs	r6, r2
	movs	r3, r1
	movs	r2, r0

	@ Shift left the denominator until it is greater than the numerator

	movs	r1, #0		@ high word of result is 0
	movs	r5, #1		@ bitmask
	adds	r7, #0		@ dont shift if denominator would overflow
	bmi	.Lldm__lower_result
	cmp	r3, r7
	blo	.Lldm__lower_result

.Lldm__denom_shift_loop4:
	lsls	r5, #1
	lsls	r7, #1
	lsls	r6, #1
	adcs	r7, r1		@ r1=0
	bmi	.Lldm__lower_result	@ dont shift if overflow
	cmp	r3, r7
	bhs	.Lldm__denom_shift_loop4



.Lldm__lower_result:
	eors	r0, r0

.Lldm__sub_loop4:
	movs	r4, r3
	cmp	r2, r6
	sbcs	r4, r7
	bcc	.Lldm__dont_sub4	@ if (num>denom)

	subs	r2, r6		@ numerator -= denom
	sbcs	r3, r7
	orrs	r0, r5		@ result(r1:r0) |= bitmask(r5)
.Lldm__dont_sub4:

	lsls	r4, r7, #31	@ denom(r7:r6) >>= 1
	lsrs	r6, #1
	lsrs	r7, #1
	orrs	r6, r4
	lsrs	r5, #1		@ bitmask(r5) >>= 1
	bne	.Lldm__sub_loop4

	pop	{r4, r5, r6, r7, pc}
