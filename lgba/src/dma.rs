//! A module allowing use of the GBA's DMA hardware.

use crate::sync::RawMutex;

static DMA0_LOCK: RawMutex = RawMutex::new();
static DMA1_LOCK: RawMutex = RawMutex::new();
static DMA2_LOCK: RawMutex = RawMutex::new();
static DMA3_LOCK: RawMutex = RawMutex::new();

