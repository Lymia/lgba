@ Emulates a few instructions present on the cortexm0, but not on the thumbv4t.
@ Bit slower, but that's how it goes on old hardware.

.macro uxth rega, regb
    lsls \rega, \regb, #16
    lsrs \rega, \rega, #16
.endm
