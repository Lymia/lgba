// This is generated code. Do not edit.
#[doc = "A terminal font supporting only 7-bit ASCII characters.\n\n                This font does not require additional storage space in the ROM, as it is used by\n                the panic handler."]
pub enum TerminalFontAscii {}
const _: () = {
    const FALLBACK_GLYPH: (u8, u16, bool) = (3usize as u8, 15usize as u16, false);
    static LO_MAP_DATA: [u16; 13usize] = [
        0u16, 0u16, 65534u16, 65535u16, 65535u16, 65535u16, 65535u16, 32767u16, 0u16, 0u16, 0u16,
        0u16, 0u16,
    ];
    static LO_MAP_HALF_DATA: [u16; 0usize] = [];
    static GLYPH_CHECK: [u16; 128usize] = [
        62529u16, 62496u16, 62510u16, 62503u16, 62557u16, 0u16, 62582u16, 62556u16, 62527u16,
        62520u16, 62508u16, 62585u16, 62536u16, 62581u16, 62544u16, 62516u16, 62568u16, 62531u16,
        62724u16, 62570u16, 62517u16, 62575u16, 62518u16, 0u16, 62558u16, 62551u16, 62506u16,
        0u16, 62546u16, 62562u16, 62514u16, 62541u16, 62553u16, 62552u16, 62523u16, 62564u16,
        62577u16, 0u16, 62554u16, 0u16, 62528u16, 62571u16, 62500u16, 62513u16, 62559u16,
        62566u16, 62540u16, 62511u16, 62584u16, 62565u16, 62542u16, 62535u16, 62728u16, 62579u16,
        62720u16, 0u16, 62501u16, 0u16, 62550u16, 62555u16, 62515u16, 62502u16, 62505u16,
        62561u16, 62548u16, 32u16, 62507u16, 0u16, 62589u16, 62549u16, 0u16, 0u16, 0u16, 62725u16,
        62538u16, 62530u16, 62498u16, 62525u16, 62576u16, 0u16, 62497u16, 62569u16, 0u16, 0u16,
        62543u16, 62588u16, 62590u16, 62583u16, 0u16, 0u16, 62563u16, 62512u16, 62545u16,
        62526u16, 62519u16, 62534u16, 62499u16, 62721u16, 62524u16, 0u16, 62532u16, 62539u16,
        62537u16, 0u16, 62573u16, 62533u16, 0u16, 0u16, 0u16, 62547u16, 0u16, 62586u16, 0u16,
        0u16, 62509u16, 62504u16, 62560u16, 62522u16, 0u16, 0u16, 62521u16, 62578u16, 62572u16,
        62574u16, 62567u16, 0u16, 62580u16, 62587u16,
    ];
    static GLYPH_ID_LO : [u8 ; 128usize] = * b"\xEF\0\xB4v\xE8\0\xA2)p25\xE1.\xE2,3&o\x01\xA5\xF2d\xB2\0\xA8j\xB5\0\xAB\xA7\xB3\xEC\xE9*q'\xE3\0\xA9\x000e7\xF3h\xA6-t\"\xE6\xACn\0c\x02\0\xF6\0\xAAis\xB6\xF5\xE7+\0u\0\xE0\xEA\0\0\0A\xAD\xAF\xB7\xF0$\0\xF7\xE5\0\0l!\xA0b\0\0g4\xEB\xB0r\xAEwB1\0/m\xED\0\xE4\xEE\0\0\0k\0\xA1\0\0\xF46(\xB1\0\0\xF1\xA3%\xA4f\0#a" ;
    static FONT_DATA : [u32 ; 448usize] = crate :: __macro_export :: xfer_u8_u32 :: < 448usize > (b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x88\x88\0\0\x88\x88\0\0\x88\x88\0\0\x88\x88\0\0\x88\x88\0\0\x88\x88\0\0\x88\x88\0\0\x88\x88\0\0\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\x88\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x000C5\x020C5\x021S5\x03\x10A\x15\0\x11Q\x15\x01\x10\x01\x11\0\x10A\x15\0\0\0\0\0\0\xB2\x0B\0d\x9A\xEB\x0C\xC4;F\0 \xEA\xAC\x02\"d\xA2\x08\xE2\x8C\xEA\x04d\xA2h\x06\0\0\0\0\0D\x88\0 \xD2-\x02\0\xBAg\x002\xBBw#\0\xBAg\0 \xD2-\x02\0D\x88\0\0\0\0\0\0\0\0\x11\0\0\x10\x01\0\0\x11\0@TE\x04\0\x11\0\0\x10\xA1\n\0\x11\xA0\n\0\0\x88\0\0\0\xFB\xBF\0\xB0O\xB4\x0B\x80H\xBE\t\x80\xF8\x97\x08\x80j\x94\t\xB0K\x94\t`\xFF\xFF\x06\0\0\0\0P\xF5\xFF\x05@\xAE\x98\x01\xE0N\xDC\x01\xA8\"\xFB\x04\xA8\x9A\xE9\x0E`\x16\xE9\x06\0v\xEF\0\0\0\0\0\0\xCC\xCC\0\xC0<\xC3\x0C\xC0<\xC3\x0C\0\xCC\xCC\x04\x80\x08\xC0\x0C\x808\xC7\x08\0\xFC\x8F\0\0\x11\0\0 \x13\x99\0\x10\xA3\x18\x01@\xECV\x05\x80\x083\0@\xFCG\x04\0\xA2\x08\0 \x12\x89\0\0\0\0\0\xA0\xFB\xBF\0\xB8G\xF4\x0B\xF8\x87\xE8\x0E\xF8\xA7\xEA\x0C\xF8\xC7\xEC\x0E\xF8\x07p\x07\xE0\xBF\xFB\x04\0\0\0\0\xE0\xFF\x7F\x06\xF0\x0F\x98\x01\xF0\x0F\x80\x08\xF0o\xF7\t\xF0\x0F\x90\t\xF0\x0F\x98\x01\xE0\xDF]\x05\0\0\0\0\xD1L\xF4\x0F\x91H\xB5\n\x91X\xA5\n\x91\xD9\xAC\n\x91X\xA5\n\xB1J\xB5\n\xD1n\xF6\r\0\0\0\0\xE6\x19q\x06\xF6\x0Ft\x07\xF6ot\x07\xF6ir\x07\xF6\tr\x07\xF6\tp\x07\xE6\x99\xF9\x0E\0\0\0\0\xA0\xFF\xFF\0\xF0\x0F\xF0\x0F\xF0\x0F\xE0\x0E\xE0\xBF\xFB\x04\xE0\x0Er\x05\xF0\x0Ft\x03\xA0_q\x06\0\0\0\0\xF1\x8E\xF8\x0Fq\x86x\x07q\x86x\x07q\x96x\x07q\x97y\x07Q\xA7{\x05\x11\xE4^\x01\0\0\0\0\xEC33\xCE\xC0\x1D\xE0\x0E\0\xDD\xEE\0\0\xF1\x0E\0\0\xFB\x8C\0\xA0[\x84\x08\xA8s7\x8A\0\0\0\0\x88dD\0\x80*F\0 \x8Af\0\"\x80l\x02\0\0\xCC\0\0\0\xC4\x08\0DD\x88\x11\x11\x11\x11 \x82\x08\0 \x02\x88\0 w\xF7\x080\x03`\x060Gd\x06p\x07`\x06 ww\x04\0\0\0\0\0 \xA2\x08\0\"\x80\x08 \xFF\xFF\t\xD0/\xD0\r\xD0o\xD4\r\xC0?\x91\t\0\xEE\xDC\t\x10\x11\x11\0\x90I&\0\x90\t\0\0\x90\xCD\xBE\x01\x90I\xB7\x08\x90Y\xA7\x08\x90I\xB7\x08\x90I\xF6\r \"\x02\0\0\x88\x08\0\0\x80\x08\0d\xB3\x7F\0t\xC7|\x07t\xC3x\x07t\xC3x\x07d\x93\xF9\x0E\0\0\0\0\0\0\0\0\0\0\0\0\xA0\xFF\xFF\x05\xF0\x0F\xE0\x0E\xE0\x1F\xD1\x0C\xA0\xCE\xDC\x05\xB0\x1BQ\x04\x80\x08@\x04\0\x88\0\0\0\x88\0\0\xF1\x8E\xF8\x0Fq\x8Ep\x07q\x9Ep\x07P\xBFs\x04\x10\xE5\xDF\x0C\0\0\0\0\0\0\x11\x01\0\x10\x01\0\xE86\xE3\x0E\xD0\x1D\xEA\x04@\xBCK\x04\x80~\xCD\x04\xA8\"\xF3\x0F\0DD\0`\xE6(\x02\"\xE0.\0\0\xC0\x0C\0\0\x80L\x04\0\xC0\x0C\0\0\xC0\x0C\0@\xC4\x08\0\0\0\0\0\x01\0\0\0\x10\0\0\x002\0\0\0 \x03\0\0\x10\0\0\0\x10\0\0\0\x01\0\0\0\0\0\0\0\x80\x04\0\0\xC0\0\0\0\xE3\x03\0\0\x85\x03\0\0\xF0\x01\0\0\xC2\x01\0\0\xB3\x06\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x0F\x0F\0\0\x0F\x0F\0\0\xC7\x07\0\0m\r\0\0=\r\0\0\0\0\0\0\0\0\0\0\x80\0\0\0\xDA\x0F\0\0\xA7\x01\0\0\xC3\x01\0\0\x92\x05\0\0\xC6\t\0\0\0\0\0\0\0\0\0\0\0\0\0\0\xFB\0\0\0\x1F\x0F\0\0\x1F\x0F\0\0\x9F\x07\0\0K\x03\0\0\0\0\0\0\x9C\x02\0\0\x84\0\0\0\x94\x06\0\0\x94\x06\0\0\xD4\x02\0\0\x96\x06\0\0\xB4\x04\0\0\0\0\0\0(\x02\0\0(\0\0\0\xFA\x06\0\0-\r\0\0y\r\0\0)\x0C\0\0|\t\0\0\0\0\0\0\x02\x08\0\0\x02\x08\0\0\xF3\x0C\0\0\x0E\x0B\0\0\x1E\x0B\0\0\x0F\x0B\0\0\xF2\r\0\0\0\0\0\0\xB1\0\0\0\x12\n\0\0\x10\0\0\0\x10\0\0\0\x10\0\0\0\x10\0\0\0U\x04\0\0\0\0\0\0c\x07\0\0A\x03\0\0i\x01\0\0\xF0\0\0\0p\x08\0\0R\0\0\0r\x06\0\0\0\0\0\0\x0F\x0F\0\0\x0F\x0F\0\0\x87\x07\0\0\xC7\x07\0\0\xC7\x07\0\0m\r\0\0=\r\0\0\0\0\0\0\xFA\x08\0\0\x87\x07\0\0\x87\x03\0\0\xE3\x01\0\0\x93\x07\0\0\x87\x07\0\0\xD2\x03\0\0\0\0\0\0\xEB\x01\0\0\x1F\x0F\0\0\x1F\x0F\0\0\x9F\x07\0\0\x0F\x07\0\0\x0F\x07\0\0K\x03\0\0\0\0\0\0\x1D\x07\0\0\x1C\x06\0\0\\\x02\0\0\\\x02\0\0\x1C\x06\0\0\x1E\x06\0\0\xBD\r\0\0\0\0\0\0{\x0B\0\0\x0F\x0C\0\0\x0F\x08\0\0\xBF\x0C\0\0\x0F\x0C\0\0\x0F\x0C\0\0[\r\0\0\0\0\0\0\xFA\0\0\0\x0F\x0F\0\0\x0F\x0B\0\0/\t\0\0\x1F\x0B\0\0\x0F\x0F\0\0\xEB\x01\0\0\0\0\0\0\xC0\0\0\0\x0E\x0C\0\0\xB9\r\0\0H\n\0\0\xF9\x01\0\0\n\0\0\0\xC0\x08\0\0\0\0\0\0\x10\0\0\0\x01\t\0\0\xE1\x01\0\0\x18\x01\0\0\x80\x01\0\0a\t\0\0\x14\0\0\0\0\0\0\0\xF5\x05\0\0\x0F\x0E\0\0\x1B\x0C\0\0\xE3\x01\0\0J\x0B\0\0K\x0B\0\0\xF0\0\0\0\0\0\0\0p\x08\0\0\x97\x0E\0\0\x18\x0E\0\0x\x08\0\0\xB8\x0C\0\0\x16\x0C\0\0s\x0B\0\0\0\0\0\0\x80\0\0\0\x08\x08\0\0\x08\x0C\0\0\xD9\t\0\0\x0C\x08\0\0\x08\x08\0\0\xA0\0\0\0\0\0\0\0\x01\0\0\0R\x02\0\0p\0\0\0v\x06\0\0p\0\0\0\xD2\x02\0\0\t\0\0\0\0\0\0\0`\x08\0\0\xC3\x02\0\0\x82\x03\0\0\xB0\0\0\0\x83\x02\0\0\x82\x01\0\0 \n\0\0\0\0\0\0\x96\x06\0\0\xDE\x0E\0\0\x1C\x04\0\0\x9C\x0C\0\0\x14\x0C\0\0\xCC\x0C\0\0\x94\x04\0\0\0\0\0\0") ;
    static GLYPH_ID_HI: [u16; 8usize] =
        [65503u16, 63355u16, 65375u16, 64815u16, 31797u16, 64755u16, 41845u16, 57148u16];
    const HI_MASK: u16 = (1 << 1usize) - 1;
    #[inline(always)]
    fn lookup_glyph(value: &u16) -> usize {
        const KEY: u32 = 1234567890u32;
        static DISPS: [u16; 16usize] = [
            1u16, 2u16, 1u16, 2u16, 4u16, 2u16, 515u16, 4u16, 772u16, 777u16, 772u16, 261u16,
            2579u16, 1575u16, 17994u16, 8805u16,
        ];
        lgba_phf::hash_u16::<16usize, 128usize>(KEY, &DISPS, value)
    }
    const CHAR_MASK: u16 = (1 << 6u32) - 1;
    fn get_font_glyph_phf(id: usize) -> (u8, u16, bool) {
        let slot = lookup_glyph(&(id as u16));
        if id == GLYPH_CHECK[slot] as usize {
            let word = GLYPH_ID_HI[slot >> 4u32];
            let hi = (word >> (1usize * (slot & 15usize))) & HI_MASK;
            let packed = (hi << 8) | (GLYPH_ID_LO[slot] as u16);
            let plane = ((packed >> 6u32) & 3) as u8;
            let char = packed & CHAR_MASK;
            let is_half = ((packed >> (6u32 + 2)) & 1) as u8;
            (plane, char, is_half != 0)
        } else {
            FALLBACK_GLYPH
        }
    }
    #[inline(never)]
    fn get_font_glyph(id: char) -> (u8, u16, bool) {
        let id = id as usize;
        if id < 208usize {
            let word = LO_MAP_DATA[id >> 4];
            if word & (1 << (id & 15)) != 0 {
                let target_i = id >> 4;
                let half_word = if target_i < LO_MAP_HALF_DATA.len() {
                    LO_MAP_HALF_DATA[target_i]
                } else {
                    0
                };
                let is_half = half_word & (1 << (id & 15)) != 0;
                ((id & 3) as u8, (id >> 2) as u16, is_half)
            } else {
                get_font_glyph_phf(id)
            }
        } else if id < 0x10000 {
            get_font_glyph_phf(id)
        } else {
            FALLBACK_GLYPH
        }
    }
    #[doc = "The data files for this font require 2.20 KiB of ROM space, not including any code specific to this font that may be generated.\n\n# Available characters\n\nThe following Unicode blocks are available in this font:\n\n* Basic Latin\n\n"]
    impl crate::display::TerminalFont for TerminalFontAscii {
        fn get_font_glyph(id: char) -> (u8, u16, bool) {
            get_font_glyph(id)
        }
        fn get_font_data() -> &'static [u32] {
            &FONT_DATA
        }
    }
    ()
};
