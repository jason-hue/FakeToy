#![feature(panic_info_message)]
#![no_std]
#![no_main]
#![feature(asm_const)]
use core::arch::global_asm;
use core::panic::PanicInfo;
use crate::tool::{memcopy, memcopy_test};

mod config;
mod console;
mod panic_handler;
mod tool;
global_asm!(include_str!("boot.S"));
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    println!("[kernel]Start executing 0x80200000!");
    memcopy_test();
    loop {}
}
