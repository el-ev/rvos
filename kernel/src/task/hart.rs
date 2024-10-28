use core::{mem::MaybeUninit, sync::atomic::AtomicBool};

use alloc::sync::Arc;
use arch::tp;
use sync::Lazy;

use crate::config::CPU_NUM;

use super::taskdef::TaskControlBlock;

// Guaranteed to be only accessed by the corresponding hart
struct HartLocal {
    #[allow(dead_code)]
    ipi_pending: AtomicBool,
    current_task: Option<Arc<TaskControlBlock>>,
}

static mut HART_LOCAL: Lazy<[HartLocal; CPU_NUM]> = Lazy::new(|| {
    let mut array = [const { MaybeUninit::uninit() }; CPU_NUM];
    for elem in array.iter_mut() {
        *elem = MaybeUninit::new(HartLocal {
            ipi_pending: AtomicBool::new(false),
            current_task: None,
        });
    }
    unsafe { core::mem::transmute::<_, [HartLocal; CPU_NUM]>(array) }
});

pub fn get_current_task() -> Option<Arc<TaskControlBlock>> {
    let hart_id = tp();
    unsafe { &HART_LOCAL[hart_id] }.current_task.clone()
}

pub fn set_current_task(task: Option<Arc<TaskControlBlock>>) {
    let hart_id = tp();
    unsafe { &mut HART_LOCAL[hart_id] }.current_task = task;
}

pub fn wake_hart(hart_id: usize) {
    // unsafe { &HART_LOCAL[hart_id] }
    //     .ipi_pending
    //     .store(true, core::sync::atomic::Ordering::Release);
    sbi::legacy::sbi_send_ipi(1 << hart_id);
}

pub fn clear_ipi() {
    // unsafe {
    //     riscv::register::sip::clear_ssoft();
    // }
    // let hart_id = tp();
    // unsafe { &HART_LOCAL[hart_id] }
    //     .ipi_pending
    //     .store(false, core::sync::atomic::Ordering::Release);
}
