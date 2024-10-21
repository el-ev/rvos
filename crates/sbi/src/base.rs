use crate::{Sbiret, sbi_call};

const EID_BASE: u64 = 0x10;

const FID_GET_SPEC_VERSION: u64 = 0;
const FID_GET_IMPL_ID: u64 = 1;
const FID_GET_IMPL_VERSION: u64 = 2;
const FID_PROBE_EXT: u64 = 3;
const FID_GET_MVENDORID: u64 = 4;
const FID_GET_MARCHID: u64 = 5;
const FID_GET_MIMPID: u64 = 6;

pub fn sbi_get_spec_version() -> Sbiret {
    sbi_call(EID_BASE, FID_GET_SPEC_VERSION, 0, 0, 0)
}

pub fn sbi_get_impl_id() -> Sbiret {
    sbi_call(EID_BASE, FID_GET_IMPL_ID, 0, 0, 0)
}

pub fn sbi_get_impl_version() -> Sbiret {
    sbi_call(EID_BASE, FID_GET_IMPL_VERSION, 0, 0, 0)
}

pub fn sbi_probe_ext() -> Sbiret {
    sbi_call(EID_BASE, FID_PROBE_EXT, 0, 0, 0)
}

pub fn sbi_get_mvendorid() -> Sbiret {
    sbi_call(EID_BASE, FID_GET_MVENDORID, 0, 0, 0)
}

pub fn sbi_get_marchid() -> Sbiret {
    sbi_call(EID_BASE, FID_GET_MARCHID, 0, 0, 0)
}

pub fn sbi_get_mimpid() -> Sbiret {
    sbi_call(EID_BASE, FID_GET_MIMPID, 0, 0, 0)
}
