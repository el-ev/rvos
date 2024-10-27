use fdt::Fdt;

use crate::config;

pub fn parse_fdt(dtb: usize) -> Fdt<'static> {
    let device_tree = unsafe { fdt::Fdt::from_ptr(dtb as _) }
        .unwrap_or_else(|e| panic!("Failed to parse device tree: {:?}", e));
    let physical_memory = device_tree
        .memory()
        .regions()
        .next()
        .expect("No memory region in device tree");
    unsafe {
        config::MEMORY_SIZE = physical_memory.size.unwrap();
    }

    // TODO: Refactor this
    let stdout = device_tree.find_compatible(&["ns16550a"]).unwrap();
    unsafe {
        config::UART_BASE = stdout.reg().unwrap().next().unwrap().starting_address as usize;
    }

    // PLIC
    let plic = device_tree.find_compatible(&["riscv,plic0"]).unwrap();
    unsafe {
        config::PLIC_BASE = plic.reg().unwrap().next().unwrap().starting_address as usize;
    }
    device_tree
}
