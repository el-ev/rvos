#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate alloc;
use alloc::vec;
use sync::Lazy;
use userlib::println;
use userlib::syscall::{syscall_env_destroy, syscall_mem_alloc, syscall_mem_map};
use userlib_macro::user_main;

const STR: &str = "Hello, world!\n\x00123123123123123123";
const PANIC_MSG: &str = "Panic!11213123";

#[user_main]
pub fn main() {
    let mut v = vec![0; 4096];
    for (i, elem) in v.iter_mut().enumerate() {
        *elem = i;
    }
    for (i, elem) in v.iter().enumerate() {
        assert_eq!(*elem, i);
    }
    // println!("test_addr");
    let _ = syscall_env_destroy(0);
}
