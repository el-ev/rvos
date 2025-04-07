use core::alloc::Layout;
use core::cmp::max;
use core::mem::size_of;
use core::ptr::NonNull;

use crate::consts::{HEAP_BEG, HEAP_SIZE, PAGE_SIZE};
use crate::syscall::{syscall_mem_alloc, syscall_mem_unmap};

use super::bitmap::Bitmap;
use super::list;

pub struct Heap<const N: usize> {
    free_area: [list::List; N],
    heap_start: usize,
    bitmap: Bitmap<{ HEAP_SIZE / PAGE_SIZE / usize::BITS as usize }>,
    total: usize,
    allocated: usize,
}

impl<const N: usize> Heap<N> {
    pub const fn new() -> Self {
        Heap {
            free_area: [list::List::new(); N],
            heap_start: HEAP_BEG,
            bitmap: Bitmap::new(),
            total: 0,
            allocated: 0,
        }
    }

    unsafe fn add_range(&mut self, mut start: usize, mut end: usize) {
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

    unsafe fn add_size(&mut self, start: usize, size: usize) {
        unsafe { self.add_range(start, start + size) };
    }

    fn add_page(&mut self, count: usize, align: usize) {
        assert!(count.is_power_of_two() && align.is_power_of_two());
        if let Some(start) = self.bitmap.find_contiguous(count, align) {
            for i in start..start + count {
                let page = self.heap_start + i * PAGE_SIZE;
                match syscall_mem_alloc(0, page, 0x3) {
                    Ok(_) => {}
                    Err(e) => {
                        panic!("Failed to allocate memory: {:?}", e);
                    }
                }
            }
            // Mark the pages as allocated in the bitmap
            self.bitmap.set(start, count);
            self.total += count * PAGE_SIZE;
            // Add the pages to the free area
            unsafe {
                self.add_size(
                    self.heap_start + start * PAGE_SIZE,
                    count * PAGE_SIZE,
                );
            }
        }
    }

    fn add_page_1(&mut self) {
        if let Some(next_zero) = self.bitmap.first_zero() {
            let page = self.heap_start + next_zero * PAGE_SIZE;
            match syscall_mem_alloc(0, page, 0x3) {
                Ok(_) => {
                    self.bitmap.set(next_zero, 1);
                    self.total += PAGE_SIZE;
                    unsafe { self.free_area[N - 1].push(page as *mut usize) }; // 11 is the order for 4kB pages
                    return;
                }
                Err(e) => {
                    panic!("Failed to allocate memory: {:?}", e);
                }
            }
        }
        panic!("User heap overflow");
    }

    pub fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let n = size.trailing_zeros() as usize;
        for i in n..N {
            if !self.free_area[i].is_empty() {
                for j in (n + 1..i + 1).rev() {
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
                    self.free_area[n]
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
        // No free block found, ask the kernel for more memory
        let page_count = size.div_ceil(PAGE_SIZE);
        self.add_page(page_count, page_count.next_power_of_two());
        // Try again
        self.alloc(layout)
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

            while order < N - 1 {
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

        // free several pages
        while let Some(free_page) = self.free_area[N - 1].pop() {
            let page = free_page as usize;
            let index = (page - self.heap_start) / PAGE_SIZE;
            syscall_mem_unmap(0, page).unwrap();
            self.bitmap.clear(index, 1);
            self.total -= PAGE_SIZE;
        }
    }

    // pub fn total(&self) -> usize {
    //     self.total
    // }

    // pub fn allocated(&self) -> usize {
    //     self.allocated
    // }
}

impl<const ORDER: usize> Default for Heap<ORDER> {
    fn default() -> Self {
        Self::new()
    }
}

fn prev_power_of_2(n: usize) -> usize {
    1 << (usize::BITS as usize - n.leading_zeros() as usize - 1)
}
