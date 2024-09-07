use core::fmt::{self, Debug};

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Context {
    pub regs: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub kernel_stval: usize,
    pub hartid: usize,
}

impl Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Context {{ regs: [")?;
        for i in 0..32 {
            write!(f, "x{}: {:#x}, ", i, self.regs[i])?;
        }
        writeln!(f, "])")?;
        writeln!(f, "sstatus: {:#x}", self.sstatus)?;
        writeln!(f, "sepc: {:#x}", self.sepc)?;
        writeln!(f, "kernel_satp: {:#x}", self.kernel_satp)?;
        writeln!(f, "kernel_sp: {:#x}", self.kernel_sp)?;
        writeln!(f, "hartid: {:#x}", self.hartid)?;
        Ok(())
    }
}
