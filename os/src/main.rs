#![no_std]
#![no_main]
use core::arch::global_asm;
use core::panic::PanicInfo;
use crate::uart::{uart_init, uart_send_string};

mod uart;
mod io;
mod config;
#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>)->!{
    loop {

    }
}
global_asm!(include_str!("boot.S"));
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    unsafe { uart_init(); }
    uart_send_string("Welcome RISC-V!\r\n");
    loop {}
}