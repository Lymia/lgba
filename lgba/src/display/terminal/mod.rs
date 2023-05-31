use crate::{
    display::{
        modes::{ActiveMode0, Mode0},
        vram::MapAccess,
        TerminalFont, VramTile,
    },
    dma::dma3,
    mmio::reg::BG_PALETTE_RAM,
    sync::{Mutex, Static},
};
use core::marker::PhantomData;

pub mod fonts;

fn update_palette(id: usize, (background, foreground): (u16, u16)) {
    let mut start = id * 0x40;
    for i in 0..4 {
        let mask = 1 << (3 - i);
        for j in start..start + 0x10 {
            let color = if (j & mask) != 0 { foreground } else { background };
            BG_PALETTE_RAM.index(j).write(color);
        }
        start += 0x10;
    }
}

/// A terminal display mode that makes it easy to display text.
pub struct Terminal {
    mode: Mode0,
    terminal_colors: [Static<(u16, u16)>; 4],
}
impl Terminal {
    pub fn new() -> Self {
        Terminal {
            mode: Mode0::new(),
            terminal_colors: [
                Static::new((0, !0)),
                Static::new((0, !0)),
                Static::new((0, !0)),
                Static::new((0, !0)),
            ],
        }
    }

    pub fn set_color(&mut self, id: usize, background: u16, foreground: u16) {
        if id >= 4 {
            terminal_color_out_of_range();
        }
        self.terminal_colors[id].write((background, foreground));
        update_palette(id, self.terminal_colors[id].read());
    }

    pub fn activate<T: TerminalFont>(&mut self) -> ActiveTerminal<T> {
        // configure all layers
        self.mode.layers[0]
            .set_enabled(true)
            .set_tile_base(28)
            .set_h_offset(4)
            .set_v_offset(4);
        self.mode.layers[1]
            .set_enabled(true)
            .set_tile_base(29)
            .set_h_offset(8)
            .set_v_offset(4);
        self.mode.layers[2]
            .set_enabled(true)
            .set_tile_base(30)
            .set_h_offset(4)
            .set_v_offset(4);
        self.mode.layers[3]
            .set_enabled(true)
            .set_tile_base(31)
            .set_h_offset(8)
            .set_v_offset(4);

        // upload font
        self.mode.layers[0]
            .char_access()
            .write_char_4bpp_dma(dma3(), 0, T::get_font_data());

        // upload palette
        for i in 0..4 {
            update_palette(i, self.terminal_colors[i].read());
        }

        // create an appropriate active terminal object
        let mut active_mode = self.mode.activate();
        let map = [
            active_mode.layers[0].map_access(0),
            active_mode.layers[1].map_access(0),
            active_mode.layers[2].map_access(0),
            active_mode.layers[3].map_access(0),
        ];
        let terminal = ActiveTerminal {
            mode: active_mode,
            terminal_colors: &self.terminal_colors,
            map,
            space_ch: [
                ActiveTerminal::<T>::tile_for_ch('\u{F508}', 0),
                ActiveTerminal::<T>::tile_for_ch('\u{F508}', 1),
                ActiveTerminal::<T>::tile_for_ch('\u{F508}', 2),
                ActiveTerminal::<T>::tile_for_ch('\u{F508}', 3),
            ],
            half_bg_ch: [
                ActiveTerminal::<T>::tile_for_ch('\u{F505}', 0),
                ActiveTerminal::<T>::tile_for_ch('\u{F505}', 1),
                ActiveTerminal::<T>::tile_for_ch('\u{F505}', 2),
                ActiveTerminal::<T>::tile_for_ch('\u{F505}', 3),
            ],
            full_bg_ch: [
                ActiveTerminal::<T>::tile_for_ch('\u{F501}', 0),
                ActiveTerminal::<T>::tile_for_ch('\u{F501}', 1),
                ActiveTerminal::<T>::tile_for_ch('\u{F501}', 2),
                ActiveTerminal::<T>::tile_for_ch('\u{F501}', 3),
            ],
            state: Mutex::new(Default::default()),
            _phantom: Default::default(),
        };

        // clear the entire terminal
        terminal.clear();

        // return the terminal
        terminal
    }
}

/// An active terminal display mode.
pub struct ActiveTerminal<'a, T: TerminalFont> {
    mode: ActiveMode0<'a>,
    terminal_colors: &'a [Static<(u16, u16)>; 4],
    map: [MapAccess; 4],

    space_ch: [VramTile; 4],
    half_bg_ch: [VramTile; 4],
    full_bg_ch: [VramTile; 4],

    state: Mutex<ActiveTerminalState>,

    _phantom: PhantomData<T>,
}
#[derive(Default)]
struct ActiveTerminalState {
    cursor_x: u8,
    cursor_y: u8,
    color: u8,
    line_advance: u8,
}
impl ActiveTerminalState {
    fn apply_advance(&self, y: usize) -> usize {
        let ny = y + (self.line_advance as usize);
        if ny >= 19 {
            ny - 19
        } else {
            ny
        }
    }
}
impl<'a, T: TerminalFont> ActiveTerminal<'a, T> {
    pub fn set_color(&self, id: usize, background: u16, foreground: u16) {
        let _lock = self.state.lock();
        if id >= 4 {
            terminal_color_out_of_range();
        }
        self.terminal_colors[id].write((background, foreground));
        update_palette(id, self.terminal_colors[id].read());
    }

    fn data_for_ch(ch: char, color: usize) -> (VramTile, bool) {
        if color >= 4 {
            terminal_color_out_of_range();
        }
        let (plane, tile, is_half) = T::get_font_glyph(ch);
        let pal = color as u8 * 4 + plane;
        (VramTile::default().with_char(tile).with_palette(pal as u8), is_half)
    }
    fn tile_for_ch(ch: char, color: usize) -> VramTile {
        Self::data_for_ch(ch, color).0
    }
    pub fn clear(&self) {
        let mut lock = self.state.lock();
        *lock = Default::default();

        let tile = self.space_ch[0];
        self.map[0].set_tile_dma(dma3(), 0, 0, tile, 32 * 32);
        self.map[1].set_tile_dma(dma3(), 0, 0, tile, 32 * 32);
        self.map[2].set_tile_dma(dma3(), 0, 0, tile, 32 * 32);
        self.map[3].set_tile_dma(dma3(), 0, 0, tile, 32 * 32);
    }

    fn check_coordinate(&self, x: usize, y: usize) {
        if x >= 58 && y >= 19 {
            terminal_coord_out_of_range();
        }
    }

    pub fn set_char(&self, x: usize, y: usize, ch: char, color: usize) {
        let lock = self.state.lock();
        let y = lock.apply_advance(y);

        self.check_coordinate(x * 2, y);

        self.map[0].set_tile(x, y, Self::tile_for_ch(ch, color));
        self.map[1].set_tile(x, y, self.space_ch[color]);
        self.map[2].set_tile(x, y, self.full_bg_ch[color]);
    }
    pub fn set_char_hw(&self, x: usize, y: usize, mut ch: char, color: usize) {
        let lock = self.state.lock();
        let y = lock.apply_advance(y);

        if ch.is_ascii() {
            ch = char::from_u32((ch as u32) + 0xF400).unwrap();
        }
        self.set_char_hw_0(x, y, ch, color);
    }
    fn set_char_hw_0(&self, x: usize, y: usize, ch: char, color: usize) -> bool {
        self.check_coordinate(x, y);

        let (plane, tile_x) = (x % 2, x / 2);
        let (tile, is_half) = Self::data_for_ch(ch, color);
        if is_half && plane == 0 {
            self.map[0].set_tile(tile_x, y, tile);
            self.map[2].set_tile(tile_x, y, self.half_bg_ch[color]);
        } else if is_half {
            self.map[1].set_tile(tile_x, y, tile);
            self.map[3].set_tile(tile_x, y, self.half_bg_ch[color]);
        } else if plane == 0 {
            self.map[0].set_tile(tile_x, y, tile);
            self.map[1].set_tile(tile_x, y, self.space_ch[color]);
            self.map[2].set_tile(tile_x, y, self.full_bg_ch[color]);
        } else {
            self.map[0].set_tile(tile_x + 1, y, self.space_ch[color]);
            self.map[1].set_tile(tile_x, y, tile);
            self.map[2].set_tile(tile_x + 1, y, self.space_ch[color]);
            self.map[3].set_tile(tile_x, y, self.full_bg_ch[color]);
        }

        is_half
    }

    pub fn write_string(&self, x: usize, y: usize, str: &str) {
        self.check_coordinate(x, y);
    }
}

#[inline(never)]
fn terminal_coord_out_of_range() -> ! {
    crate::panic_handler::static_panic(
        "Terminal coordinates are from (0,0) to (57,18) inclusive in half-width mode and between\
        (0,0) to (28,18) inclusive in full-width mode.",
    )
}

#[inline(never)]
fn terminal_color_out_of_range() -> ! {
    crate::panic_handler::static_panic("Terminals only support up to 4 colors (0-3)!")
}
