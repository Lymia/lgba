    .section .header.meta, "ax", %progbits
    .arm
    .global __lgba_header_extra
__lgba_header_extra:
    @ Multiplay header
    b __lgba_multiplay_start
    .space 0x1C
    b __lgba_joybus_start @ joybus entry point; not currently supported

    @ lgba metainfo header
    .ascii "lGex"         @ lGex header
    .ascii "meta"         @
    .short 0              @ header version
    .short 20             @ length
0:  .word 0b
    .word __lgba_exh_rom_cname
    .word __lgba_exh_rom_cver
    .word __lgba_exh_rom_repository
    .word __lgba_exh_lgba_version


    .section .lgba.header.extra_end, "ax", %progbits
    .arm
    .global __lgba_header_extra_end
__lgba_header_extra_end:
    .ascii "exh_"