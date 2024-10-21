use core::marker::PhantomData;
use core::{fmt, ptr};

#[derive(Copy, Clone)]
pub struct List {
    head: *mut usize,
}

unsafe impl Send for List {}

pub struct Node {
    prev: *mut usize,
    curr: *mut usize,
}

impl List {
    pub const fn new() -> Self {
        List {
            head: ptr::null_mut(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub unsafe fn push(&mut self, elm: *mut usize) {
        unsafe {
            *elm = self.head as usize;
        }
        self.head = elm;
    }

    pub fn pop(&mut self) -> Option<*mut usize> {
        if self.is_empty() {
            None
        } else {
            let elm = self.head;
            unsafe {
                self.head = *elm as *mut usize;
            }
            Some(elm)
        }
    }

    pub fn iter(&self) -> Iter {
        Iter {
            curr: self.head,
            _marker: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            curr: self.head,
            _marker: PhantomData,
        }
    }
}

impl Node {
    pub fn value(&self) -> *mut usize {
        self.curr
    }

    pub fn pop(self) -> *mut usize {
        unsafe {
            *self.prev = *self.curr;
            self.curr
        }
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub struct Iter<'a> {
    curr: *mut usize,
    _marker: PhantomData<&'a List>,
}

pub struct IterMut<'a> {
    prev: *mut usize,
    curr: *mut usize,
    _marker: PhantomData<&'a mut List>,
}

impl Iterator for Iter<'_> {
    type Item = *mut usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let item = self.curr;
            unsafe {
                self.curr = *item as *mut usize;
            }
            Some(item)
        }
    }
}

impl Iterator for IterMut<'_> {
    type Item = Node;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr.is_null() {
            None
        } else {
            let node = Node {
                prev: self.prev,
                curr: self.curr,
            };
            self.prev = self.curr;
            unsafe {
                self.curr = *self.curr as *mut usize;
            }
            Some(node)
        }
    }
}
