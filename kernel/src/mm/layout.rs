use log::info;

extern "C" {
    pub fn __kernel_start();
    pub fn __kernel_end();
    pub fn __text_start();
    pub fn __text_end();
    pub fn __rodata_start();
    pub fn __rodata_end();
    pub fn __data_start();
    pub fn __data_end();
    pub fn __bss_start();
    pub fn __bss_end();
}

pub fn print_memory_layout() {
    let kernel_start = __kernel_start as usize;
    let kernel_end = __kernel_end as usize;
    let text_start = __text_start as usize;
    let text_end = __text_end as usize;
    let rodata_start = __rodata_start as usize;
    let rodata_end = __rodata_end as usize;
    let data_start = __data_start as usize;
    let data_end = __data_end as usize;
    let bss_start = __bss_start as usize;
    let bss_end = __bss_end as usize;

    let kernel_size_mb = (kernel_end - kernel_start) / 1024 / 1024;
    info!(
        r#"Kernel memory layout:
  .text   : 0x{:x} - 0x{:x}
  .rodata : 0x{:x} - 0x{:x}
  .data   : 0x{:x} - 0x{:x}
  .bss    : 0x{:x} - 0x{:x}
  Kernel  : 0x{:x} - 0x{:x} ({} MB)  "#,
        text_start,
        text_end,
        rodata_start,
        rodata_end,
        data_start,
        data_end,
        bss_start,
        bss_end,
        kernel_start,
        kernel_end,
        kernel_size_mb
    )
}
