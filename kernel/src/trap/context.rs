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
        writeln!(f, "UserContext {{")?;
        writeln!(f, "  uregs: [")?;
        for i in 0..32 {
            if i % 4 == 0 {
                if i != 0 {
                    writeln!(f)?;
                }
                write!(f, "    ")?;
            }
            write!(f, "x{:<2}: {:#18x}, ", i, self.uregs[i])?;
        }
        writeln!(f, "],")?;
        writeln!(f, "  usstatus: {:#x},", self.usstatus)?;
        writeln!(f, "  sepc: {:#x},", self.sepc)?;
        writeln!(f, "  ksregs: [")?;
        for i in 0..12 {
            if i % 4 == 0 {
                if i != 0 {
                    writeln!(f)?;
                }
                write!(f, "    ")?;
            }
            write!(f, "s{:<2}: {:#18x}, ", i, self.ksregs[i])?;
        }
        writeln!(f, "],")?;
        writeln!(f, "  kra: {:#x},", self.kra)?;
        writeln!(f, "  ksp: {:#x},", self.ksp)?;
        writeln!(f, "  ktp: {:#x}", self.ktp)?;
        writeln!(f, "}}")
    }
}
