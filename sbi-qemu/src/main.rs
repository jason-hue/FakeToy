#![no_std]
#![no_main]
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::csr::{insert_field, MSTATUS_MPIE, MSTATUS_MPP, PRV_S};
use config::FW_JUMP_ADDR;
mod config;
mod panic_handler;
mod csr;
mod pmp;
global_asm!(include_str!("sbi_boot.S"));
/*
 * 运行在M模式
 */
#[no_mangle]
pub extern "C" fn sbi_main() -> ! {
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
    unsafe {
        asm!("mret");
        core::hint::unreachable_unchecked();
    }
}