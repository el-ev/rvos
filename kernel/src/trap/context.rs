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

impl UserContext {}

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
    pub regs: [usize; 16], // ra, t0-2, a0-7, t3-6
    pub sepc: usize,
    pub stval: usize,
}

impl Debug for KernelContext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "KernelContext {{ regs: [")?;
        write!(f, "ra: {:#x}, ", self.regs[0])?;
        write!(f, "t0: {:#x}, ", self.regs[1])?;
        write!(f, "t1: {:#x}, ", self.regs[2])?;
        write!(f, "t2: {:#x}, ", self.regs[3])?;
        write!(f, "a0: {:#x}, ", self.regs[4])?;
        write!(f, "a1: {:#x}, ", self.regs[5])?;
        write!(f, "a2: {:#x}, ", self.regs[6])?;
        write!(f, "a3: {:#x}, ", self.regs[7])?;
        write!(f, "a4: {:#x}, ", self.regs[8])?;
        write!(f, "a5: {:#x}, ", self.regs[9])?;
        write!(f, "a6: {:#x}, ", self.regs[10])?;
        write!(f, "a7: {:#x}, ", self.regs[11])?;
        write!(f, "t3: {:#x}, ", self.regs[12])?;
        write!(f, "t4: {:#x}, ", self.regs[13])?;
        write!(f, "t5: {:#x}, ", self.regs[14])?;
        write!(f, "t6: {:#x}, ", self.regs[15])?;
        writeln!(f, "])")?;
        writeln!(f, "sepc: {:#x}", self.sepc)?;
        writeln!(f, "stval: {:#x}", self.stval)?;
        Ok(())
    }
}
