use crate::mmio::{
    reg::{TM_CNT_H, TM_CNT_L},
    sys::TimerCnt,
};

pub fn temp_time(func: impl FnOnce()) {
    TM_CNT_L.index(0).write(0);
    TM_CNT_L.index(1).write(0);
    TM_CNT_H
        .index(0)
        .write(TimerCnt::default().with_enabled(true));
    TM_CNT_H
        .index(1)
        .write(TimerCnt::default().with_enabled(true).with_cascade(true));

    func();

    let cycles = TM_CNT_L.index(0).read() as u32 | ((TM_CNT_L.index(1).read() as u32) << 16);

    TM_CNT_H.index(0).write(TimerCnt::default());
    TM_CNT_H.index(1).write(TimerCnt::default());

    crate::println!("Function took {} cycles.", cycles);
}
