use crate::{
    display::{
        modes::{ActiveMode0, Mode0},
        vram::MapAccess,
        TerminalFont, VramTile,
    },
    dma::dma3,
    mmio::reg::BG_PALETTE_RAM,
};
use core::marker::PhantomData;

pub mod fonts;

fn update_palette(id: usize, (background, foreground): (u16, u16)) {
    let mut start = id * 0x40;
    for i in 0..4 {
        let mask = 1 << (3-i);
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
    terminal_colors: [(u16, u16); 4],
}
impl Terminal {
    pub fn new() -> Self {
        Terminal { mode: Mode0::new(), terminal_colors: [(0, !0); 4] }
    }

    pub fn set_color(&mut self, id: usize, background: u16, foreground: u16) {
        if id >= 4 {
            terminal_color_out_of_range();
        }
        self.terminal_colors[id] = (background, foreground);
        update_palette(0, self.terminal_colors[id]);
    }

    pub fn activate<T: TerminalFont>(&mut self) -> ActiveTerminal<T> {
        // configure all layers
        self.mode.layers[0].set_enabled(true).set_tile_base(29);
        self.mode.layers[1]
            .set_enabled(false) // TODO
            .set_tile_base(30)
            .set_h_offset(4);

        // upload font
        self.mode.layers[0]
            .char_access()
            .write_char_4bpp_dma(dma3(), 0, T::get_font_data());

        // upload palette
        for i in 0..4 {
            update_palette(i, self.terminal_colors[i]);
        }

        // create an approprate active terminal
        let mut active_mode = self.mode.activate();
        let map = active_mode.layers[0].map_access(0);
        let terminal = ActiveTerminal {
            mode: active_mode,
            terminal_colors: &mut self.terminal_colors,
            map,
            _phantom: Default::default(),
        };

        // clear the entire terminal
        for x in 0..32 {
            for y in 0..32 {
                terminal.set_char(x, y, ' ', 0);
            }
        }

        // return the terminal
        terminal
    }
}

/// An active terminal display mode.
pub struct ActiveTerminal<'a, T: TerminalFont> {
    mode: ActiveMode0<'a>,
    terminal_colors: &'a mut [(u16, u16); 4],
    map: MapAccess,
    _phantom: PhantomData<T>,
}
impl<'a, T: TerminalFont> ActiveTerminal<'a, T> {
    pub fn set_color(&mut self, id: usize, background: u16, foreground: u16) {
        if id >= 4 {
            terminal_color_out_of_range();
        }
        self.terminal_colors[id] = (background, foreground);
        update_palette(0, self.terminal_colors[id]);
    }

    pub fn set_char(&self, x: usize, y: usize, ch: char, color: usize) {
        if color >= 4 {
            terminal_color_out_of_range();
        }
        let (plane, tile, _) = T::get_font_glyph(ch);
        let pal = color as u8 * 4 + plane;

        self.map
            .set_tile(x, y, VramTile::default().with_char(tile).with_palette(pal as u8))
    }
}

#[inline(never)]
fn terminal_color_out_of_range() -> ! {
    crate::panic_handler::static_panic("Terminals only support up to 4 colors (0-3)!")
}
