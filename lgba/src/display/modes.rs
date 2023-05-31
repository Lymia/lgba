use crate::{
    display::layers::{ActiveTileLayer, LayerId, TileLayer},
    mmio::{display::DispMode, reg::DISPCNT},
    sync::{RawMutex, RawMutexGuard},
};

static MAIN_GFX_LOCK: RawMutex = RawMutex::new();

#[derive(Debug)]
#[non_exhaustive]
pub struct Mode0 {
    pub layers: [TileLayer; 4],
}
impl Mode0 {
    pub fn new() -> Self {
        Mode0 {
            layers: [
                TileLayer::new(LayerId::Layer0),
                TileLayer::new(LayerId::Layer1),
                TileLayer::new(LayerId::Layer2),
                TileLayer::new(LayerId::Layer3),
            ],
        }
    }

    pub fn activate(&mut self) -> ActiveMode0 {
        let [layer0, layer1, layer2, layer3] = &mut self.layers;
        let new_disp_cnt = DISPCNT
            .read()
            .with_mode(DispMode::Mode0)
            .with_forced_blank(false)
            .with_display_bg0(layer0.enabled())
            .with_display_bg1(layer1.enabled())
            .with_display_bg2(layer2.enabled())
            .with_display_bg3(layer3.enabled());
        let active_mode = ActiveMode0 {
            layers: [layer0.activate(), layer1.activate(), layer2.activate(), layer3.activate()],
            lock: MAIN_GFX_LOCK
                .try_lock()
                .unwrap_or_else(|| graphics_in_use()),
        };
        DISPCNT.write(new_disp_cnt);
        active_mode
    }
}

pub struct ActiveMode0<'a> {
    pub layers: [ActiveTileLayer<'a>; 4],
    lock: RawMutexGuard<'static>,
}
impl<'a> Drop for ActiveMode0<'a> {
    fn drop(&mut self) {
        // force blank when there's no active graphics mode
        DISPCNT.write(DISPCNT.read().with_forced_blank(true));
    }
}

#[inline(never)]
fn graphics_in_use() -> ! {
    crate::panic_handler::static_panic("A graphics mode is already activated!")
}