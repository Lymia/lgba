use crate::sync::{Mutex, Static};
use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    ops::Range,
    ptr,
    ptr::NonNull,
};
use linked_list_allocator::Heap;

static IWRAM_HEAP: Mutex<Heap> = Mutex::new(Heap::empty());
static EWRAM_HEAP: Mutex<Heap> = Mutex::new(Heap::empty());

static IWRAM_ALLOC: Static<usize> = Static::new(0);
static EWRAM_ALLOC: Static<usize> = Static::new(0);
static IWRAM_SIZE: Static<usize> = Static::new(0);
static EWRAM_SIZE: Static<usize> = Static::new(0);

fn check_interrupt() {
    if crate::irq::is_in_interrupt() {
        allocator_in_interrupt();
    }
}

#[inline(never)]
#[track_caller]
const fn allocator_in_interrupt() {
    panic!("Cannot allocate memory in an interrupt.");
}

fn map_layout(l: Layout) -> Layout {
    // should help the allocator with how simple it is
    if l.align() < 4 {
        Layout::from_size_align(l.size(), 4).unwrap()
    } else {
        l
    }
}
fn alloc(
    heap: &mut Heap,
    alloc: &Static<usize>,
    layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    let result = heap.allocate_first_fit(layout);
    match result {
        Ok(v) => {
            alloc.write(alloc.read() + layout.size());
            Ok(NonNull::slice_from_raw_parts(v, layout.size()))
        }
        Err(_) => Err(AllocError),
    }
}
unsafe fn dealloc(heap: &mut Heap, alloc: &Static<usize>, ptr: NonNull<u8>, layout: Layout) {
    alloc.write(alloc.read() - layout.size());
    heap.deallocate(ptr, layout);
}
fn iwram_alloc(layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    alloc(&mut IWRAM_HEAP.lock(), &IWRAM_SIZE, layout)
}
fn ewram_alloc(layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    alloc(&mut EWRAM_HEAP.lock(), &EWRAM_SIZE, layout)
}

/// Allocator for iwram.
#[derive(Copy, Clone, Default)]
pub struct Iwram;
unsafe impl Allocator for Iwram {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        check_interrupt();
        let layout = map_layout(layout);
        iwram_alloc(layout)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        check_interrupt();
        let layout = map_layout(layout);
        dealloc(&mut IWRAM_HEAP.lock(), &IWRAM_ALLOC, ptr, layout);
    }
}

/// Allocator for ewram.
#[derive(Copy, Clone, Default)]
pub struct Ewram;
unsafe impl Allocator for Ewram {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        check_interrupt();
        let layout = map_layout(layout);
        ewram_alloc(layout)
    }
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        check_interrupt();
        let layout = map_layout(layout);
        dealloc(&mut EWRAM_HEAP.lock(), &EWRAM_ALLOC, ptr, layout);
    }
}

struct DefaultAlloc;
unsafe impl GlobalAlloc for DefaultAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        check_interrupt();
        let layout = map_layout(layout);
        let result = if layout.size() <= 64 {
            iwram_alloc(layout).or_else(|_| ewram_alloc(layout))
        } else {
            ewram_alloc(layout).or_else(|_| iwram_alloc(layout))
        };
        match result {
            Ok(v) => v.as_ptr().as_mut_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        check_interrupt();
        let layout = map_layout(layout);
        if let Some(ptr) = NonNull::new(ptr) {
            let addr = (ptr.as_ptr() as usize);
            if addr > 0x2000000 && addr < 0x2010000 {
                dealloc(&mut IWRAM_HEAP.lock(), &IWRAM_ALLOC, ptr, layout);
            } else if addr > 0x3000000 && addr < 0x2040000 {
                dealloc(&mut EWRAM_HEAP.lock(), &EWRAM_ALLOC, ptr, layout);
            } else {
                dealloc_invalid_address();
            }
        }
    }
}

#[inline(never)]
#[track_caller]
const fn dealloc_invalid_address() {
    panic!("Attempt to deallocate invalid address.");
}

#[global_allocator]
static ALLOC: DefaultAlloc = DefaultAlloc;

extern "C" {
    static __heap_start: u8;
}

/// Returns the total amount of bytes of heap available in iwram.
#[inline(always)]
pub fn iwram_capacity() -> usize {
    IWRAM_SIZE.read()
}

/// Returns the number of bytes of heap that are already allocated in iwram.
#[inline(always)]
pub fn iwram_used() -> usize {
    IWRAM_ALLOC.read()
}

/// Returns the total amount of bytes of heap available in ewram.
#[inline(always)]
pub fn ewram_capacity() -> usize {
    EWRAM_SIZE.read()
}

/// Returns the number of bytes of heap that are already allocated in ewram.
#[inline(always)]
pub fn ewram_used() -> usize {
    EWRAM_ALLOC.read()
}

/// Returns the total amount of bytes of heap available.
#[inline(always)]
pub fn heap_capacity() -> usize {
    IWRAM_SIZE.read() + EWRAM_SIZE.read()
}

/// Returns the number of bytes of heap that are already allocated.
#[inline(always)]
pub fn heap_used() -> usize {
    IWRAM_ALLOC.read() + EWRAM_ALLOC.read()
}

fn align_alloc(range: Range<usize>) -> Range<usize> {
    ((range.start + 63) & !63)..range.end
}
pub(crate) unsafe fn init_rust_alloc() {
    let iwram = align_alloc(crate::asm::iwram_alloc_range());
    let ewram = align_alloc(crate::asm::ewram_alloc_range());

    IWRAM_HEAP.lock().init(iwram.start as *mut u8, iwram.len());
    EWRAM_HEAP.lock().init(ewram.start as *mut u8, ewram.len());

    IWRAM_SIZE.write(iwram.len());
    EWRAM_SIZE.write(ewram.len());
}
