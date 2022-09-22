/// A helper type used to write character data into VRAM.
#[derive(Debug)]
pub struct CharAccess {
    base: usize,
    lower_bound: usize,
    upper_bound: usize,
}
impl CharAccess {
    pub(crate) fn new(base: usize, lower_bound: usize, upper_bound: usize) -> Self {
        CharAccess { base, lower_bound, upper_bound }
    }
}
