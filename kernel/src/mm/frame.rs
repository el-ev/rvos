use core::fmt;

use alloc::vec::Vec;
use log::{info, warn};
use sync::SpinNoIrqMutex;

use crate::prev_pow_of_2;

use super::{
    addr::{PhysAddr, PhysPageNum},
    consts::FRAME_SIZE,
};

const ORDER: usize = 32;

pub static FRAME_ALLOCATOR: SpinNoIrqMutex<FrameAllocator<ORDER>> =
    SpinNoIrqMutex::new(FrameAllocator::new());

#[derive(Debug, Clone)]
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        FrameTracker { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        dealloc(self.ppn);
    }
}

pub struct FrameAllocator<const ORDER: usize> {
    free_list: [Vec<PhysPageNum>; ORDER],
    total: usize,
    allocated: usize,
}
impl<const ORDER: usize> FrameAllocator<ORDER> {
    const fn new() -> Self {
        FrameAllocator {
            free_list: [const { Vec::new() }; ORDER],
            total: 0,
            allocated: 0,
        }
    }

    pub fn init(&mut self, start: PhysAddr, end: PhysAddr) {
        debug_assert!(start.0 <= end.0);
        let start = start.ceil_page();
        let end = end.floor_page();
        let mut current = start;
        while current < end {
            let lowbit = 1 << current.0.trailing_zeros();
            let size = usize::min(lowbit, prev_pow_of_2!(end.0 - current.0));
            let order = size.trailing_zeros() as usize;
            self.free_list[order].push(current);
            current += size;
        }
        self.total = end.0 - start.0;
    }

    pub fn alloc(&mut self, size: usize, align: usize) -> Option<PhysPageNum> {
        if size == 0 || align == 0 || size > self.total || align > self.total {
            return None;
        }
        debug_assert!(size.is_power_of_two());
        debug_assert!(align.is_power_of_two());
        let order = size.trailing_zeros() as usize;
        let align_order = align.trailing_zeros() as usize;
        let start_order = usize::max(order, align_order);
        for i in start_order..ORDER {
            if !self.free_list[i].is_empty() {
                for j in ((order + 1)..=i).rev() {
                    let ppn = self.free_list[j]
                        .pop()
                        .expect("There should be some free frames");
                    self.free_list[j - 1].push(PhysPageNum(ppn.0 + (1 << (j - 1)))); // This is the buddy frame
                    self.free_list[j - 1].push(ppn); // This is the allocated frame, which matches the align
                }
                let ppn = self.free_list[order]
                    .pop()
                    .expect("There should be some free frames");
                self.allocated += 1 << order;
                return Some(ppn);
            }
        }
        None
    }

    pub fn dealloc(&mut self, frame: PhysPageNum) {
        let mut ppn = frame;
        let mut order = 0;
        while order < ORDER - 1 {
            let buddy = PhysPageNum(ppn.0 ^ (1 << order));
            let mut found = false;
            for block in &self.free_list[order] {
                if *block == buddy {
                    found = true;
                    break;
                }
            }
            if found {
                self.free_list[order].retain(|x| *x != buddy);
                ppn = PhysPageNum(ppn.0 & buddy.0);
                order += 1;
            } else {
                break;
            }
        }
        self.free_list[order].push(ppn);
        self.allocated -= 1;
    }
}

impl fmt::Debug for FrameAllocator<ORDER> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "FrameAllocator {{")?;
        writeln!(f, "  total: {}, allocated: {}", self.total, self.allocated)?;
        writeln!(f, "  free_list: [")?;
        for i in 0..ORDER {
            if !self.free_list[i].is_empty() {
                write!(f, "    order {:2}: ", i)?;
                for ppn in &self.free_list[i] {
                    write!(f, "{} ", ppn)?;
                }
                writeln!(f)?;
            } else {
                writeln!(f, "    order {:2}: empty", i)?;
            }
        }
        writeln!(f, "  ]")?;
        writeln!(f, "}}")
    }
}

unsafe fn clear_frame(frame: PhysPageNum, size: usize) {
    let ptr = PhysAddr::from(frame).as_mut_ptr::<u8>();
    ptr.write_bytes(0, FRAME_SIZE * size);
}

pub fn debug_print() {
    log::debug!("\n{:#?}", FRAME_ALLOCATOR.lock());
}

pub fn init(start: PhysAddr, end: PhysAddr) {
    FRAME_ALLOCATOR.lock().init(start, end);
    info!(
        "Initialized frame allocator with {} frames in total.",
        FRAME_ALLOCATOR.lock().total
    );
}

pub fn alloc_frames(size: usize, align: usize) -> Option<Vec<FrameTracker>> {
    let frame = FRAME_ALLOCATOR.lock().alloc(size, align);
    if let Some(frame) = frame {
        unsafe {
            clear_frame(frame, size);
        }
    } else {
        warn!(
            "Failed to allocate {} frames with alignment {}.",
            size, align
        );
    }
    frame.map(|frame| {
        (0..size)
            .map(|i| FrameTracker::new(PhysPageNum(frame.0 + i)))
            .collect()
    })
}
pub fn alloc() -> Option<FrameTracker> {
    alloc_frames(1, 1).map(|mut v| v.pop().unwrap())
}

pub fn dealloc(frame: PhysPageNum) {
    FRAME_ALLOCATOR.lock().dealloc(frame);
}
