use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicBool, AtomicUsize},
};

use alloc::{format, string::String};
use arch::get_hart_id;
use log::error;

use crate::{mm::layout::{__bss_end, __data_end, __text_end, __text_start}, task};

static PANIC_HAPPENING: AtomicBool = AtomicBool::new(false);
static PANIC_HART: AtomicUsize = AtomicUsize::new(0);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
    if PANIC_HAPPENING
        .compare_exchange(
            false,
            true,
            core::sync::atomic::Ordering::Release,
            core::sync::atomic::Ordering::Relaxed,
        )
        .is_err()
        && PANIC_HART.load(core::sync::atomic::Ordering::Relaxed) != arch::tp()
    {
        loop {}
    }
    PANIC_HART.store(arch::tp(), core::sync::atomic::Ordering::Relaxed);
    unsafe {
        crate::console::poison_lock();
    }
    let cur_task = task::hart::get_current_task();
    if let Some(location) = info.location() {
        if let Some(task) = cur_task {
            error!(
                "\x1b[1;31mPanicked: \"{}\" from hart {} at {}:{} in task {:?}\n Context: {:?}{}\x1b[1;0m",
                info.message(),
                get_hart_id(),
                location.file(),
                location.line(),
                task.pid(),
                task.get_context(),
                backtrace()
            );
        } else {
            error!(
                "\x1b[1;31mPanicked: \"{}\" from hart {} at {}:{}{}\x1b[1;0m",
                info.message(),
                get_hart_id(),
                location.file(),
                location.line(),
                backtrace()
            );
        }
    } else {
        // error!(
        //     "\x1b[1;31mPanicked: {} from hart {}{}\x1b[1;0m",
        //     info.message(),
        //     get_hart_id(),
        //     backtrace()
        // );
        if let Some(task) = cur_task {
            error!(
                "\x1b[1;31mPanicked: \"{}\" from hart {} in task {:?}\n Context: {:?}{}\x1b[1;0m",
                info.message(),
                get_hart_id(),
                task.pid(),
                task.get_context(),
                backtrace()
            );
        } else {
            error!(
                "\x1b[1;31mPanicked: \"{}\" from hart {}{}\x1b[1;0m",
                info.message(),
                get_hart_id(),
                backtrace()
            );
        }
    }
    sbi::reset::sbi_shutdown()
}

fn backtrace() -> String {
    let mut result = String::new();
    #[cfg(feature = "print_symbol")]
    let symbols = read_symbol();
    let mut depth = 0;
    let mut current_ra = arch::ra();
    let mut current_fp = arch::fp();
    result.push_str("\nBacktrace:\n");
    while current_ra >= __text_start as usize
        && current_ra <= __text_end as usize
        && current_fp >= __data_end as usize
        && current_fp <= __bss_end as usize
    {
        #[cfg(feature = "print_symbol")]
        {
            let mut here: &str = "unknown";
            for ((start, end), name) in &symbols {
                if current_ra >= *start && current_ra <= *end {
                    here = name;
                    break;
                }
            }
            result.push_str(&format!(
                "  {:02}: RA = 0x{:016x}, FP = 0x{:016x} at {:#}\n",
                depth,
                current_ra - size_of::<usize>(),
                current_fp,
                rustc_demangle::demangle(here)
            ));
        }
        #[cfg(not(feature = "print_symbol"))]
        {
            result.push_str(&format!(
                "  {:02}: RA = 0x{:016x}, FP = 0x{:016x}\n",
                depth,
                current_ra - size_of::<usize>(),
                current_fp
            ));
        }
        unsafe {
            current_ra = *(current_fp as *const usize).sub(1);
            current_fp = *(current_fp as *const usize).sub(2);
        }
        depth += 1;
    }
    if depth == 0 {
        result.push_str("  <no backtrace>\n");
    }
    result
}

#[cfg(feature = "print_symbol")]
fn read_symbol() -> alloc::vec::Vec<((usize, usize), &'static str)> {
    use xmas_elf::{
        sections::{self, ShType},
        symbol_table::Entry,
    };

    use crate::include_bytes_align_as;

    let mut symbols = alloc::vec::Vec::new();
    // TODO: file system
    #[cfg(debug_assertions)]
    let elf = xmas_elf::ElfFile::new(
        include_bytes_align_as!(
            usize,
            "../../target/riscv64gc-unknown-none-elf/debug/kernel"
        )
        .as_ref(),
    )
    .unwrap();
    #[cfg(not(debug_assertions))]
    let elf = xmas_elf::ElfFile::new(
        include_bytes_align_as!(
            usize,
            "../../target/riscv64gc-unknown-none-elf/release/kernel"
        )
        .as_ref(),
    )
    .unwrap();
    for sect in elf.section_iter() {
        if sect.get_type() == Ok(ShType::SymTab) {
            if let Ok(sections::SectionData::SymbolTable64(data)) = sect.get_data(&elf) {
                for sym in data {
                    if sym.get_type().unwrap() != xmas_elf::symbol_table::Type::Func {
                        continue;
                    }
                    let name = elf.get_string(sym.name()).unwrap();
                    if name.starts_with("$") || name.starts_with(".L") {
                        continue;
                    }
                    symbols.push((
                        (
                            sym.value() as usize,
                            sym.value() as usize + sym.size() as usize,
                        ),
                        name,
                    ));
                }
            }
        }
    }
    symbols
}
