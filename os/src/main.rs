#![feature(panic_info_message)]
#![no_std]
#![no_main]
use core::arch::global_asm;
use core::panic::PanicInfo;
mod config;
mod console;
mod panic_handler;
global_asm!(include_str!("boot.S"));
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    println!("Welcome RISC-V!");
    loop {}
}