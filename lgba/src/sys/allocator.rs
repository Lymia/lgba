use crate::sync::{Mutex, Static};
use core::{
    alloc::{AllocError, GlobalAlloc, Layout},
    ops::Range,
    ptr,
    ptr::NonNull,
};
use linked_list_allocator::Heap;

// TODO: Implement `Allocator`s

type HeapInfo = (Range<usize>, Heap);

static HEAP_ZONES: Mutex<Option<&'static mut [HeapInfo]>> = Mutex::new(None);
static HEAP_ALLOC: Static<usize> = Static::new(0);
static HEAP_SIZE: Static<usize> = Static::new(0);

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
fn alloc(heap: &mut Heap, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    let result = heap.allocate_first_fit(layout);
    match result {
        Ok(v) => {
            HEAP_ALLOC.write(HEAP_ALLOC.read() + layout.size());
            Ok(NonNull::slice_from_raw_parts(v, layout.size()))
        }
        Err(_) => Err(AllocError),
    }
}
unsafe fn dealloc(heap: &mut Heap, ptr: NonNull<u8>, layout: Layout) {
    HEAP_ALLOC.write(HEAP_ALLOC.read() - layout.size());
    heap.deallocate(ptr, layout);
}

struct DefaultAlloc;
unsafe impl GlobalAlloc for DefaultAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        check_interrupt();
        let layout = map_layout(layout);

        let mut zones = HEAP_ZONES.lock();
        let zones = zones.as_mut().unwrap();
        for (_, zone) in zones.iter_mut() {
            match alloc(zone, layout) {
                Ok(v) => return v.as_ptr().as_mut_ptr(),
                Err(_) => {}
            }
        }
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        check_interrupt();
        if !ptr.is_null() {
            let layout = map_layout(layout);

            let mut zones = HEAP_ZONES.lock();
            let zones = zones.as_mut().unwrap();
            for (range, zone) in zones.iter_mut() {
                if range.contains(&(ptr as usize)) {
                    dealloc(zone, NonNull::new_unchecked(ptr), layout);
                    return;
                }
            }
            dealloc_invalid_address();
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

/// Returns the total amount of bytes of heap available.
#[doc(cfg(feature = "allocator"))]
#[inline(always)]
pub fn heap_capacity() -> usize {
    HEAP_SIZE.read()
}

/// Returns the number of bytes of heap that are already allocated.
#[doc(cfg(feature = "allocator"))]
#[inline(always)]
pub fn heap_used() -> usize {
    HEAP_ALLOC.read()
}

fn check_range_validity(range: Range<usize>) -> Range<usize> {
    if range.end < range.start {
        range.start..range.start
    } else {
        range
    }
}
fn align_array(range: Range<usize>) -> Range<usize> {
    check_range_validity(((range.start + 7) & !7)..range.end)
}
fn align_alloc(range: Range<usize>) -> Range<usize> {
    check_range_validity(((range.start + 63) & !63)..range.end)
}
fn find_control_zone(layout: Layout, zones: &[Range<usize>]) -> usize {
    // try to find an appropriate iwram zone
    for (i, zone) in zones.iter().enumerate() {
        if layout.size() <= align_array(zone.clone()).len() && zone.start >= 0x3000000 {
            return i;
        }
    }

    // try to find an appropriate zone
    for (i, zone) in zones.iter().enumerate() {
        if layout.size() <= align_array(zone.clone()).len() {
            return i;
        }
    }

    // opps
    panic!("could not find control zone for allocation")
}
pub(crate) unsafe fn init_rust_alloc() {
    crate::asm::alloc_zones(|zones| {
        let required_size = Layout::array::<HeapInfo>(zones.len()).unwrap();
        let control_zone = find_control_zone(required_size, zones);
        let control_range = align_array(zones[control_zone].clone());

        let start_range = control_range.start as *mut HeapInfo;
        for i in 0..zones.len() {
            let range = if i == control_zone {
                align_alloc(control_range.start + required_size.size()..control_range.end)
            } else {
                align_alloc(zones[i].clone())
            };
            ptr::write(
                start_range.offset(i as isize),
                (range.clone(), Heap::new(range.start as *mut u8, range.len())),
            );
        }

        let slice = core::slice::from_raw_parts_mut(start_range, zones.len());
        *HEAP_ZONES.lock() = Some(slice);
    });
}
