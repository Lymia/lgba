MEMORY {
    rom (rx)    : ORIGIN = 0x08000000, LENGTH = 32M
    ewram (w!x) : ORIGIN = 0x02000000, LENGTH = 256K
    iwram (w!x) : ORIGIN = 0x03000000, LENGTH = 32K
}

ENTRY  (__start)
INPUT  (liblgba_as.a)

SECTIONS {
    .text : {
        KEEP(*(.lgba.header));
        *(.lgba.header);
        *(.text .text.*);
        . = ALIGN(4);
    } >rom = 0xff

    .rodata : {
        *(.rodata .rodata.*);
        . = ALIGN(4);
    } >rom = 0xff

    .bss : {
        __bss_start = ABSOLUTE(.);
        *(.bss .bss.*);
        . = ALIGN(4);
        __bss_end = ABSOLUTE(.);
        . = ALIGN(4);
    } >iwram

    __iwram_lma = .;
    .iwram : {
        __iwram_start = ABSOLUTE(.);
        *(.data .data.*);
        *(.iwram .iwram.*);
        . = ALIGN(4);
        __iwram_end = ABSOLUTE(.);
        . = ALIGN(4);
    } >iwram

    __ewram_lma = .;
    .ewram : {
        __ewram_start = ABSOLUTE(.);
        *(.ewram .ewram.*);
        . = ALIGN(4);
        __ewram_end = ABSOLUTE(.);
        . = ALIGN(4);
        __heap_start = ABSOLUTE(.);
    } >ewram

    /* DWARF 1 */
    .debug          0 : { *(.debug) }
    .line           0 : { *(.line) }
    /* GNU DWARF 1 extensions */
    .debug_srcinfo  0 : { *(.debug_srcinfo) }
    .debug_sfnames  0 : { *(.debug_sfnames) }
    /* DWARF 1.1 and DWARF 2 */
    .debug_aranges  0 : { *(.debug_aranges) }
    .debug_pubnames 0 : { *(.debug_pubnames) }
    /* DWARF 2 */
    .debug_info     0 : { *(.debug_info) }
    .debug_abbrev   0 : { *(.debug_abbrev) }
    .debug_line     0 : { *(.debug_line) }
    .debug_frame    0 : { *(.debug_frame) }
    .debug_str      0 : { *(.debug_str) }
    .debug_loc      0 : { *(.debug_loc) }
    .debug_macinfo  0 : { *(.debug_macinfo) }
    .debug_ranges   0 : { *(.debug_ranges) }
    /* SGI/MIPS DWARF 2 extensions */
    .debug_weaknames 0 : { *(.debug_weaknames) }
    .debug_funcnames 0 : { *(.debug_funcnames) }
    .debug_typenames 0 : { *(.debug_typenames) }
    .debug_varnames  0 : { *(.debug_varnames) }

    /* discard anything not already mentioned */
    /DISCARD/ : { *(*) }
}
