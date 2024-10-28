use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::btree_map::BTreeMap, sync::Arc};
use arch::SIEGuard;
use log::{debug, trace, warn};
use riscv::interrupt::{
    Trap,
    supervisor::{Exception, Interrupt},
};
use sync::Lazy;

use crate::{
    Mutex, config,
    error::OsError,
    get_hart_count,
    mm::paging::switch_page_table,
    syscall,
    task::user_space::UserPageFaultType,
    timer,
    trap::{context::UserContext, set_kernel_trap, set_user_trap},
};

use super::{
    hart::{clear_ipi, get_current_task, set_current_task, wake_hart},
    pid::Pid,
    taskdef::{TaskControlBlock, TaskStatus},
};

pub static SCHEDULER: Lazy<Scheduler> = Lazy::new(Scheduler::new);

pub struct Scheduler {
    tasks: Mutex<BTreeMap<Pid, Arc<TaskControlBlock>>>,
    queue: Mutex<[Option<Arc<TaskControlBlock>>; config::MAX_TASKS]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    alive_task_count: AtomicUsize,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(BTreeMap::new()),
            queue: Mutex::new([const { None }; config::MAX_TASKS]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            alive_task_count: AtomicUsize::new(0),
        }
    }

    pub fn submit_task(&self, task: Arc<TaskControlBlock>) -> Result<(), OsError> {
        let mut tail = self.tail.load(Ordering::Relaxed);
        loop {
            let next_tail = (tail + 1) % config::MAX_TASKS;
            if next_tail == self.head.load(Ordering::Relaxed) {
                return Err(OsError::NoFreeTask);
            }

            match self.tail.compare_exchange_weak(
                tail,
                next_tail,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.tasks.lock().insert(task.pid(), task.clone());
                    self.queue.lock()[tail] = Some(task);
                    self.alive_task_count.fetch_add(1, Ordering::Release);
                    return Ok(());
                }
                Err(x) => tail = x,
            }
        }
    }

    fn try_get_task(&self) -> Option<Arc<TaskControlBlock>> {
        let mut head = self.head.load(Ordering::Relaxed);
        loop {
            if head == self.tail.load(Ordering::Relaxed) {
                return None;
            }

            match self.head.compare_exchange_weak(
                head,
                (head + 1) % config::MAX_TASKS,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let task = self.queue.lock()[head].take();
                    return task;
                }
                Err(x) => head = x,
            }
        }
    }

    fn return_task(&self, task: Arc<TaskControlBlock>) {
        let mut tail = self.tail.load(Ordering::Relaxed);
        loop {
            let next_tail = (tail + 1) % config::MAX_TASKS;
            if next_tail == self.head.load(Ordering::Relaxed) {
                panic!("Task queue is full, should not happen");
            }

            match self.tail.compare_exchange_weak(
                tail,
                next_tail,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.queue.lock()[tail] = Some(task);

                    let target_hart = (tail + 1) % get_hart_count();
                    wake_hart(target_hart);
                    return;
                }
                Err(x) => tail = x,
            }
        }
    }

    pub fn hart_loop(&self) -> ! {
        loop {
            clear_ipi();
            if self.head.load(Ordering::Acquire) == self.tail.load(Ordering::Acquire) {
                if self.alive_task_count.load(Ordering::Acquire) == 0 {
                    panic!("No task to run");
                }
                riscv::asm::wfi();
            }

            match self.try_get_task() {
                Some(task) => loop {
                    self.execute(task.clone());
                    if task.is_exited() {
                        debug!("Task {:?} exited, runs: {}", task.pid(), task.runs());
                        task.do_exit();
                        self.tasks.lock().remove(&task.pid());
                        self.alive_task_count
                            .fetch_sub(1, core::sync::atomic::Ordering::Release);
                        break;
                    }
                    if self.head.load(Ordering::Acquire) != self.tail.load(Ordering::Acquire) {
                        self.return_task(task);
                        break;
                    }
                },
                None => continue,
            }
        }
    }

    fn execute(&self, task: Arc<TaskControlBlock>) {
        // debug!("Hart {} is running task {:?}", arch::tp(), task.pid());
        let sie_guard = SIEGuard::new();
        if task.status() != TaskStatus::Ready {
            panic!("Task {:?} is not ready, should not happen", task.pid());
        }
        let current_task = get_current_task();
        if !(current_task.is_some() && current_task.unwrap().pid() == task.pid()) {
            switch_page_table(task.page_table().ppn());
            set_current_task(Some(task.clone()));
        }
        timer::set_next_timeout();
        drop(sie_guard);

        set_user_trap();
        unsafe {
            _kernel_to_user(task.get_context_ptr());
        }
        set_kernel_trap();

        let sie_guard = SIEGuard::new();
        task.inc_runs();
        let scause = riscv::register::scause::read().cause().try_into().unwrap();
        match scause {
            Trap::Interrupt(i) => match i {
                Interrupt::SupervisorTimer => timer::set_next_timeout(),
                Interrupt::SupervisorSoft => {},
                Interrupt::SupervisorExternal => todo!(),
            },
            Trap::Exception(e) => match e {
                Exception::UserEnvCall => syscall::do_syscall(task.clone()),
                Exception::LoadPageFault
                | Exception::StorePageFault
                | Exception::InstructionPageFault => {
                    let stval = riscv::register::stval::read();
                    let ty = match e {
                        Exception::LoadPageFault => UserPageFaultType::Read,
                        Exception::StorePageFault => UserPageFaultType::Write,
                        Exception::InstructionPageFault => UserPageFaultType::Execute,
                        _ => unreachable!(),
                    };
                    trace!(
                        "User page fault, pid: {:?}, stval: {:#x}",
                        task.pid(),
                        stval,
                    );
                    match task.memory().lock().handle_page_fault(stval, ty) {
                        Ok(()) => {}
                        Err(_) => {
                            warn!(
                                "User page fault, killed. Pid: {:?}, sepc: {:#x}, stval: {:#x}",
                                task.pid(),
                                task.get_context().sepc,
                                stval,
                            );
                            task.exit();
                        }
                    }
                }
                Exception::IllegalInstruction
                | Exception::InstructionFault
                | Exception::InstructionMisaligned => {
                    warn!(
                        "User Illegal instruction, killed. Pid: {:?}, sepc: {:#x}",
                        task.pid(),
                        task.get_context().sepc,
                    );
                    task.exit();
                }
                _ => {
                    panic!(
                        "Unhandled exception: {:?}, pid: {:?}, context: {:?}",
                        e,
                        task.pid(),
                        task.get_context()
                    );
                }
            },
        };
        drop(sie_guard);
    }
}

unsafe extern "C" {
    fn _kernel_to_user(ctx: *mut UserContext);
}
