use core::arch::asm;
use core::ptr::write_volatile;
use crate::config::VIRT_CLINT_TIMER_CMP;
use crate::csr::insert_field;
use crate::{read_csr, write_csr};
use crate::trap::trap_handler::{MIE_MTIE, MIP_MTIP, MIP_STIP};

pub fn clint_timer_event_start(next_event: usize){
    unsafe { writeq(next_event, VIRT_CLINT_TIMER_CMP as *mut usize); }//把下次预设的定时器值写入到MTIMECMP寄存器中
    let mie = read_csr!("mie");
    let new_mie = insert_field(mie, MIE_MTIE as u32, 1);
    write_csr!("mie", new_mie);
}
pub fn sbi_timer_process(){
    //清除 MTIP 位
    let mie = read_csr!("mie");
    let new_mie = insert_field(mie, MIP_MTIP as u32, 0);
    write_csr!("mie", new_mie);

    //设置 STIP 位，触发 S 模式定时器中断
    let mip = read_csr!("mip");
    let new_mip = insert_field(mip, MIP_STIP as u32, 1);
    write_csr!("mip", new_mip);
}
#[inline(always)]
pub unsafe fn writeq(value: usize, addr: *mut usize) {
    fence();
    write_volatile(addr, value);
}//向目标地址写入一个64的值
#[inline(always)]
pub fn fence() {
    unsafe {
        asm!("fence iorw, iorw");
    }
}