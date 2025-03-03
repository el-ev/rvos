use core::{
    fmt::{self, Display},
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::{
    mask,
    mm::consts::{PAGE_SIZE, PAGE_SIZE_BITS, VA_WIDTH},
    round_up,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub usize);

impl Add<usize> for VirtAddr {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtAddr {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for VirtAddr {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for VirtAddr {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VA(0x{:x})", self.0)
    }
}

impl VirtAddr {
    pub fn ceil(self) -> Self {
        Self(round_up!(self.0, PAGE_SIZE))
    }

    pub fn floor(self) -> Self {
        Self(self.0 & mask!(PAGE_SIZE))
    }

    pub fn offset(self) -> usize {
        self.0 & mask!(PAGE_SIZE_BITS)
    }

    pub fn ceil_page(self) -> VirtPageNum {
        VirtPageNum(round_up!(self.0, PAGE_SIZE) >> PAGE_SIZE_BITS)
    }

    pub fn floor_page(self) -> VirtPageNum {
        VirtPageNum(self.0 >> PAGE_SIZE_BITS)
    }

    pub fn as_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    pub unsafe fn as_mut_page_slice(&self) -> &'static mut [u8] {
        unsafe { self.as_mut_slice(PAGE_SIZE) }
    }

    pub unsafe fn as_mut_slice(&self, len: usize) -> &'static mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.as_mut_ptr(), len) }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct VirtPageNum(pub usize);

impl Add<usize> for VirtPageNum {
    type Output = Self;
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for VirtPageNum {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl Sub<usize> for VirtPageNum {
    type Output = Self;
    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign<usize> for VirtPageNum {
    fn sub_assign(&mut self, rhs: usize) {
        self.0 -= rhs;
    }
}

impl Display for VirtPageNum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VPN(0x{:x})", self.0)
    }
}

impl From<VirtAddr> for usize {
    fn from(v: VirtAddr) -> Self {
        v.0
    }
}
impl From<usize> for VirtAddr {
    fn from(v: usize) -> Self {
        Self(v & mask!(VA_WIDTH))
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(v: VirtAddr) -> Self {
        v.floor_page()
    }
}
impl From<VirtPageNum> for VirtAddr {
    fn from(v: VirtPageNum) -> Self {
        Self(v.0 << PAGE_SIZE_BITS)
    }
}

impl From<VirtPageNum> for usize {
    fn from(v: VirtPageNum) -> Self {
        v.0
    }
}
