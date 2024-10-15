use core::panic::PanicInfo;

use alloc::{format, string::String};
use arch::get_hart_id;
use log::error;

use crate::mm::layout::{__text_end, __text_start};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        riscv::register::sstatus::clear_sie();
    }
    if let Some(location) = info.location() {
        error!(
            "\x1b[1;31mPanicked: \"{}\" from hart {} at {}:{}{}\x1b[1;0m",
            info.message(),
            get_hart_id(),
            location.file(),
            location.line(),
            backtrace()
        );
    } else {
        error!(
            "\x1b[1;31mPanicked: {} from hart {}{}\x1b[1;0m",
            info.message(),
            get_hart_id(),
            backtrace()
        );
    }
    sbi::reset::sbi_shutdown()
}

fn backtrace() -> String {
    let mut result = String::new();
    #[cfg(feature = "print_symbol")]
    let symbols = read_symbol();
    unsafe {
        let mut current_ra = arch::ra();
        let mut current_fp = arch::fp();
        let mut depth = 0;
        result.push_str("\nBacktrace:\n");
        while current_ra >= __text_start as usize
            && current_ra <= __text_end as usize
            && current_fp != 0
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
            current_ra = *(current_fp as *const usize).sub(1);
            current_fp = *(current_fp as *const usize).sub(2);
            depth += 1;
        }
    }
    result
}

#[cfg(feature = "print_symbol")]
fn read_symbol() -> alloc::collections::btree_map::BTreeMap<(usize, usize), &'static str> {
    use xmas_elf::{
        sections::{self, ShType},
        symbol_table::Entry,
    };

    let mut symbols = alloc::collections::btree_map::BTreeMap::new();
    // TODO: file system
    #[cfg(debug_assertions)]
    let elf = xmas_elf::ElfFile::new(
        include_bytes!("../../target/riscv64gc-unknown-none-elf/debug/kernel").as_ref(),
    )
    .unwrap();
    #[cfg(not(debug_assertions))]
    let elf = ElfFile::new(
        include_bytes!("../../target/riscv64gc-unknown-none-elf/release/kernel").as_ref(),
    )
    .unwrap();
    for sect in elf.section_iter() {
        if sect.get_type() == Ok(ShType::SymTab) {
            if let Some(sections::SectionData::SymbolTable64(data)) = sect.get_data(&elf).ok() {
                for sym in data {
                    let name = elf.get_string(sym.name()).unwrap();
                    if name.starts_with("$") || name.starts_with(".L") {
                        continue;
                    }
                    symbols.insert(
                        (
                            sym.value() as usize,
                            sym.value() as usize + sym.size() as usize,
                        ),
                        name,
                    );
                }
            }
        }
    }
    symbols
}
