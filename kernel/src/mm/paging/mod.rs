use log::debug;

use crate::entry::BOOT_PAGE_TABLE;

use super::addr::PhysPageNum;
use super::consts::PAGE_SIZE;

pub mod page_table;
pub mod pte;

#[inline]
pub fn switch_page_table(pt: PhysPageNum) -> PhysPageNum {
    let old_pt = riscv::register::satp::read().ppn();
    if old_pt == pt.0 {
        return PhysPageNum(old_pt);
    }
    unsafe {
        riscv::register::satp::set(riscv::register::satp::Mode::Sv39, 0, pt.0);
        riscv::asm::sfence_vma_all();
    }
    debug!("Switched page table to 0x{:x}", pt.0 * PAGE_SIZE);
    PhysPageNum(old_pt)
}

pub fn flush_tlb(vaddr: usize) {
    unsafe { riscv::asm::sfence_vma(0, vaddr) };
}

pub fn flush_tlb_all() {
    unsafe { riscv::asm::sfence_vma_all() };
}

pub fn unmap_low_memory() {
    unsafe {
        BOOT_PAGE_TABLE[..256].fill(pte::PageTableEntry::EMPTY);
    }
    flush_tlb_all();
}
