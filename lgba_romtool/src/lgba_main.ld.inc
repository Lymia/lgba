SECTIONS {
    __root_lma = .;

    /* ROM headers */
    .header : {
        /* The fixed early part of the header, including the JMP instruction and logo */
        KEEP(*(.lgba.header.fixed));
        *(.lgba.header.fixed);

        /* The dynamic part of the header, generated from the #[entry] attribute */
        KEEP(*(.lgba.header.dynamic));
        *(.lgba.header.dynamic);

        /* Extra headers used by lgba, to allow post-processing of the ROM */
        KEEP(*(.header .header.*));
        *(.header .header.*);

        /* The end marker for the extra headers */
        KEEP(*(.lgba.header.extra_end));
        *(.lgba.header.extra_end);
        . = ALIGN(4);
    } >rom

    /* The text segment, stored on ROM */
    .text : {
        *(.text .text.*);
        . = ALIGN(4);
    } >rom

    /* The read-only segment, stored on ROM */
    .rodata : {
        /* The list of constructors called by lgba's init function. */
        . = ALIGN(4);
        __ctor_start = ABSOLUTE(.);
        KEEP(*(.ctor .ctor.*));
        *(.ctor .ctor.*);
        __ctor_end = ABSOLUTE(.);
        . = ALIGN(4);

        /* Rest of the read-only data */
        *(.rodata .rodata.*);
        . = ALIGN(16);
    } >rom

    /* Data stored in EWRAM. This is stored in ROM initially and copied into RAM. */
    __ewram_lma = .;
    .ewram : {
        __ewram_start = ABSOLUTE(.);
        *(.ewram .ewram.*); /* explicit ewram variables */
        . = ALIGN(4);
    } >ewram AT>rom
    .ewram_text : {
        *(.ewram_text .ewram_text.*);
        . = ALIGN(16);
        __ewram_end = ABSOLUTE(.); /* explicit ewram code */
    } >ewram AT>rom

    /* Data stored in IWRAM. This is stored in ROM initially and copied into RAM. */
    . = __ewram_lma + (__ewram_end - __ewram_start);
    __iwram_lma = .;
    .iwram : {
        __iwram_start = ABSOLUTE(.);
        *(.data .data.*); /* iwram is default storage location for variables */
        *(.iwram .iwram.*); /* explicit iwram variables */
        . = ALIGN(4);
    } >iwram AT>rom
    .iwram_text : {
        *(.iwram_text .iwram_text.*); /* explicit iwram code */
        . = ALIGN(4);
        __iwram_end = ABSOLUTE(.);
    } >iwram AT>rom

    /* The part of IWRAM that is zeroed instead of having any initial contents */
    .bss : {
        __bss_start = ABSOLUTE(.);
        *(.bss .bss.*);
        . = ALIGN(16);
        __bss_end = ABSOLUTE(.);
    } >iwram

    /* Dynamic info */
    .dynamic   : { *(.dynamic) } >rom
    .dynsym    : { *(.dynsym) } >rom
    .dynstr    : { *(.dynstr) } >rom

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

    /* required sections */
    .shstrtab  0 : { *(.shstrtab) }

    /* discard anything not already mentioned */
    /DISCARD/ : { *(*) }
}
