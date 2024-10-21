use core::alloc::Layout;
use core::cmp::max;
use core::mem::size_of;
use core::ptr::NonNull;

use crate::list;

pub struct Heap<const ORDER: usize> {
    free_area: [list::List; ORDER],

    total: usize,
    allocated: usize,
}

impl<const ORDER: usize> Heap<ORDER> {
    pub const fn new() -> Self {
        Heap {
            free_area: [list::List::new(); ORDER],
            total: 0,
            allocated: 0,
        }
    }

    pub unsafe fn add_range(&mut self, mut start: usize, mut end: usize) {
        // align start and end
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end &= !size_of::<usize>() + 1;
        debug_assert!(start <= end);
        let mut total = 0;
        while start + size_of::<usize>() <= end {
            // This ensures the memory is aligned.
            let lowbit = 1 << start.trailing_zeros();
            let size = lowbit.min(prev_power_of_2(end - start));

            total += size;
            unsafe { self.free_area[size.trailing_zeros() as usize].push(start as *mut usize) };
            start += size;
        }
        self.total += total;
    }

    pub unsafe fn add_size(&mut self, start: usize, size: usize) {
        unsafe { self.add_range(start, start + size) };
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let order = size.trailing_zeros() as usize;
        for i in order..ORDER {
            if !self.free_area[i].is_empty() {
                for j in (order + 1..i + 1).rev() {
                    if let Some(block) = self.free_area[j].pop() {
                        unsafe {
                            // the "left" block is pushed later, so it's popped first
                            self.free_area[j - 1]
                                .push((block as usize + (1 << (j - 1))) as *mut usize);
                            self.free_area[j - 1].push(block);
                        }
                    } else {
                        return Err(());
                    }
                }
                let result = NonNull::new(
                    self.free_area[order]
                        .pop()
                        .expect("the block should be there") as *mut u8,
                );
                if let Some(result) = result {
                    self.allocated += size;
                    return Ok(result);
                } else {
                    return Err(());
                }
            }
        }
        Err(())
    }

    pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let order = size.trailing_zeros() as usize;
        unsafe {
            self.free_area[order].push(ptr.as_ptr() as *mut usize);

            // Check for free buddy
            let mut order = order;
            let mut block = ptr.as_ptr() as usize;

            while order < ORDER - 1 {
                let mut flag = false;
                let buddy = block ^ (1 << order);
                for block in self.free_area[order].iter_mut() {
                    if block.value() as usize == buddy {
                        block.pop();
                        flag = true;
                        break;
                    }
                }
                if flag {
                    self.free_area[order].pop();
                    block &= buddy;
                    order += 1;
                    self.free_area[order].push(block as *mut usize);
                } else {
                    break;
                }
            }
        }
        self.allocated -= size;
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn allocated(&self) -> usize {
        self.allocated
    }
}

impl<const ORDER: usize> Default for Heap<ORDER> {
    fn default() -> Self {
        Self::new()
    }
}

fn prev_power_of_2(n: usize) -> usize {
    1 << (usize::BITS as usize - n.leading_zeros() as usize - 1)
}
