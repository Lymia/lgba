ENTRY  ({start_symbol})

MEMORY {{
    ewram (w!x) : ORIGIN = 0x{ewram_origin:x}, LENGTH = 0x{ewram_len:x}
    iwram (w!x) : ORIGIN = 0x{iwram_origin:x}, LENGTH = 0x{iwram_len:x}
    rom (rx)    : ORIGIN = 0x{rom_origin:x}, LENGTH = 0x{rom_len:x}
}}

{rest}
