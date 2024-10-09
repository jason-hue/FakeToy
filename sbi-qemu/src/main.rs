#![feature(panic_info_message)]
#![no_std]
#![no_main]
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::csr::{insert_field, MSTATUS_MPIE, MSTATUS_MPP, PRV_S};
use config::FW_JUMP_ADDR;
use crate::config::logo;
use crate::pmp::{init_pmp, print_pmp_info};
use crate::trap::sbi_trap_init;
use crate::uart::{uart_init, uart_send};

mod config;
mod panic_handler;
mod csr;
mod pmp;
mod trap;
mod console;
mod uart;
global_asm!(include_str!("sbi_boot.S"));
global_asm!(include_str!("trap/trap.S"));
/*
 * 运行在M模式
 */
#[no_mangle]
pub extern "C" fn sbi_main() -> ! {
    uart_init();
    println!("SBI: UART INIT!");
    println!("{}",logo);
    println!("SBI: Initializing...");
    /* 初始化PMP*/
    init_pmp();
    print_pmp_info();
    /* 设置跳转模式为S模式 */
    let mut val = read_csr!("mstatus");
    val = insert_field(val,MSTATUS_MPP,PRV_S);
    val = insert_field(val,MSTATUS_MPIE,0);
    write_csr!("mstatus",val);
    /* 设置M模式的Exception Program Counter，用于mret跳转 */
    write_csr!("mepc",FW_JUMP_ADDR);
    /* 设置S模式异常向量表入口*/
    write_csr!("stvec", FW_JUMP_ADDR);
    /* 关闭S模式的中断*/
    write_csr!("sie", 0);
    /* 关闭S模式的页表转换 */
    write_csr!("satp", 0);
    println!("SBI: mstatus = 0x{:016x}", read_csr!("mstatus"));
    println!("SBI: mepc = 0x{:016x}", read_csr!("mepc"));
    println!("SBI: stvec = 0x{:016x}", read_csr!("stvec"));
    println!("SBI: satp = 0x{:016x}", read_csr!("satp"));
    sbi_trap_init();
    println!("SBI: SBI_TRAP_INIT!");
    unsafe {
        println!("SBI: Executing mret");
        asm!("mret");
        println!("SBI: This should not be printed");
        core::hint::unreachable_unchecked();
    }
}