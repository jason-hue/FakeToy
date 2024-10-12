#![feature(panic_info_message)]
#![no_std]
#![no_main]
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::csr::{read_sepc, read_sstatus};
use crate::sbi::{putchar, sbi_call};
use crate::trap::trap_init;

mod config;
mod console;
mod panic_handler;
mod csr;
mod sbi;
mod trap;
global_asm!(include_str!("boot.S"));
global_asm!(include_str!("trap/trap.S"));
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    trap_init();
    println!("Entering kernel_main");
    println!("About to execute invalid instruction");
    println!("If you see this, trap handling worked correctly");
    loop {
        println!("System running normally");
        // 添加一些延迟
        for _ in 0..1000000 { core::hint::spin_loop(); }
    }
}