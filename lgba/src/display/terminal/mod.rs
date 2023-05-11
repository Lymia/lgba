use crate::{
    display::{
        modes::{ActiveMode0, Mode0},
        TerminalFont, VramTile,
    },
    dma::dma3,
    mmio::reg::BG_PALETTE_RAM,
};
use core::marker::PhantomData;

pub mod fonts;

/// A terminal display mode that makes it easy to display text.
pub struct Terminal {
    mode: Mode0,
}
impl Terminal {
    pub fn new() -> Self {
        Terminal { mode: Mode0::new() }
    }

    pub fn activate<T: TerminalFont>(&mut self) -> ActiveTerminal<T> {
        // configure all layers
        self.mode.layers[0].set_enabled(true).set_tile_base(29);
        self.mode.layers[1]
            .set_enabled(true)
            .set_tile_base(30)
            .set_h_offset(4);

        // upload font
        self.mode.layers[0]
            .char_access()
            .write_char_4bpp_dma(dma3(), 0, T::get_font_data());

        // TODO: Temporary debug
        self.mode.layers[0]
            .map_access(0)
            .set_tile(0, 0, VramTile::default().with_char(1));

        for i in 0..255 {
            BG_PALETTE_RAM.index(i).write((i * 1088442) as u16);
        }

        // return an approprate active terminal
        let mut active_mode = self.mode.activate();
        ActiveTerminal { mode: active_mode, _phantom: Default::default() }
    }
}

/// An active terminal display mode.
pub struct ActiveTerminal<'a, T: TerminalFont> {
    mode: ActiveMode0<'a>,
    _phantom: PhantomData<T>,
}
impl<'a, T: TerminalFont> ActiveTerminal<'a, T> {}
