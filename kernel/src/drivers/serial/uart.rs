use crate::print;

use super::ConsoleDevice;

const RBR: usize = 0x00; // Receive Buffer Register (Read Only)
const THR: usize = 0x00; // Transmit Holding Register (Write Only)
const DLL: usize = 0x00; // Divisor Latch (Least Significant Byte)
const DLM: usize = 0x01; // Divisor Latch (Most Significant Byte)
const IER: usize = 0x01; // Interrupt Enable Register
const FCR: usize = 0x02; // FIFO Control Register
const LCR: usize = 0x03; // Line Control Register
const MCR: usize = 0x04; // Modem Control Register
const LSR: usize = 0x05; // Line Status Register
const MSR: usize = 0x06; // Modem Status Register
const SCR: usize = 0x07; // Scratchpad Register
const MDR1: usize = 0x08; // Multi-Function Data Register 1

const LSR_THRE: u8 = 0x20; // Transmit Holding Register Empty
const LSR_DR: u8 = 0x01; // Data Ready

pub struct Uart {
    base: usize,
    freq: u32,
    baudrate: u32,
    reg_width: usize,
    reg_shift: usize,
}

macro_rules! read_reg {
    ($ptr:expr, $reg:expr, $shift:expr) => {
        $ptr.byte_add($reg << $shift).read_volatile()
    };
}

macro_rules! write_reg {
    ($ptr:expr, $reg:expr, $val:expr, $shift:expr) => {
        $ptr.byte_add($reg << $shift).write_volatile($val);
    };
}

impl Uart {
    pub fn new(base: usize, freq: u32, baudrate: u32, reg_width: usize, reg_shift: usize) -> Self {
        Self {
            base,
            freq,
            baudrate,
            reg_width,
            reg_shift,
        }
    }
}

impl ConsoleDevice for Uart {
    fn init(&self) {
        match self.reg_width {
            1 => {
                let ptr = self.base as *mut u8;
                unsafe {
                    write_reg!(ptr, IER, 0x00, self.reg_shift);
                    write_reg!(ptr, LCR, 0x80, self.reg_shift);
                    let divisor = self.freq / (16 * self.baudrate);
                    write_reg!(ptr, DLL, divisor as u8, self.reg_shift);
                    write_reg!(ptr, DLM, (divisor >> 8) as u8, self.reg_shift);
                    write_reg!(ptr, LCR, 0x03, self.reg_shift);
                    write_reg!(ptr, FCR, 0x01, self.reg_shift);
                    write_reg!(ptr, MCR, 0x00, self.reg_shift);
                    write_reg!(ptr, IER, 0x01, self.reg_shift);
                }
            },
            4 => {
                let ptr = self.base as *mut u32;
                unsafe {
                    write_reg!(ptr, IER, 0x00, self.reg_shift);
                    write_reg!(ptr, LCR, 0x80, self.reg_shift);
                    let divisor = self.freq / (16 * self.baudrate);
                    write_reg!(ptr, DLL, divisor, self.reg_shift);
                    write_reg!(ptr, DLM, divisor >> 8, self.reg_shift);
                    write_reg!(ptr, LCR, 0x03, self.reg_shift);
                    write_reg!(ptr, FCR, 0x01, self.reg_shift);
                    write_reg!(ptr, MCR, 0x00, self.reg_shift);
                    write_reg!(ptr, IER, 0x01, self.reg_shift);
                }
            },
            _ => panic!("Invalid register width"),
        }
    }

    fn putc(&self, c: u8) {
        match self.reg_width {
            1 => {
                let ptr = self.base as *mut u8;
                unsafe {
                    while read_reg!(ptr, LSR, self.reg_shift) & LSR_THRE == 0 {}
                    write_reg!(ptr, THR, c, self.reg_shift);
                }
            },
            4 => {
                let ptr = self.base as *mut u32;
                unsafe {
                    while read_reg!(ptr, LSR, self.reg_shift) & LSR_THRE as u32 == 0 {}
                    write_reg!(ptr, THR, c as u32, self.reg_shift);
                }
            },
            _ => panic!("Invalid register width"),
        }
    }

    fn try_getc(&self) -> Option<u8> {
        match self.reg_width {
            1 => {
                let ptr = self.base as *mut u8;
                unsafe {
                    if read_reg!(ptr, LSR, self.reg_shift) & LSR_DR != 0 {
                        Some(read_reg!(ptr, RBR, self.reg_shift))
                    } else {
                        None
                    }
                }
            },
            4 => {
                let ptr = self.base as *mut u32;
                unsafe {
                    if read_reg!(ptr, LSR, self.reg_shift) & LSR_DR as u32 != 0 {
                        Some(read_reg!(ptr, RBR, self.reg_shift) as u8)
                    } else {
                        None
                    }
                }
            },
            _ => panic!("Invalid register width"),
        }
    }
}