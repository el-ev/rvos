use core::fmt::{self, Debug};

#[repr(C)]
#[derive(Default)]
pub struct UserContext {
    pub uregs: [usize; 32], // 0-31
    pub usstatus: usize,    // 32
    pub sepc: usize,        // 33

    pub ksregs: [usize; 12], // 34-45
    pub kra: usize,          // 46
    pub ksp: usize,          // 47
    pub ktp: usize,          // 48
}

impl Debug for UserContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "UserContext {{ uregs: [")?;
        for i in 0..32 {
            write!(f, "x{}: {:#x}, ", i, self.uregs[i])?;
        }
        writeln!(f, "])")?;
        writeln!(f, "usstatus: {:#x}", self.usstatus)?;
        writeln!(f, "sepc: {:#x}", self.sepc)?;
        writeln!(f, "ksregs: [")?;
        for i in 0..12 {
            write!(f, "s{}: {:#x}, ", i, self.ksregs[i])?;
        }
        writeln!(f, "])")?;
        writeln!(f, "kra: {:#x}", self.kra)?;
        writeln!(f, "ksp: {:#x}", self.ksp)?;
        writeln!(f, "ktp: {:#x}", self.ktp)?;
        Ok(())
    }
}

#[repr(C)]
pub struct KernelContext {
    pub regs: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub kernel_sp: usize,
    pub kernel_stval: usize,
    pub hartid: usize,
}

impl Debug for KernelContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "KernelContext {{ regs: [")?;
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
