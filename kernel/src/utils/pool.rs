use alloc::vec::Vec;

pub struct UsizePool {
    next: usize,
    recycled: Vec<usize>,
}

impl UsizePool {
    pub const fn new() -> Self {
        Self {
            next: 0,
            recycled: Vec::new(),
        }
    }

    pub fn alloc(&mut self) -> usize {
        if let Some(id) = self.recycled.pop() {
            id
        } else {
            let id = self.next;
            self.next += 1;
            id
        }
    }

    pub fn dealloc(&mut self, id: usize) {
        self.recycled.push(id);
    }
}