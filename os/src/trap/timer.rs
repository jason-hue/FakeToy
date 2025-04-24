use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};
use crate::config::{CLINT_TIMEBASE_FREQ, HZ, SIE_STIE, VIRT_CLINT_TIMER_VAL};
use crate::csr::{insert_field, SSTATUS_SIE, SSTATUS_SPP};
use crate::{println, read_csr, write_csr};
use crate::sbi::sbi_set_timer;
use core::sync::atomic::{AtomicUsize, Ordering};

#[repr(align(64))]
struct AlignedAtomicUsize(AtomicUsize);

static JIFFIES: AlignedAtomicUsize = AlignedAtomicUsize(AtomicUsize::new(0));

// 使用时需要访问内部的AtomicUsize
pub fn get_jiffies() -> usize {
    JIFFIES.0.load(Ordering::Relaxed)
}

pub fn increment_jiffies() {
    JIFFIES.0.fetch_add(1, Ordering::Relaxed);
}


pub fn handle_timer_irq(){
    //关闭s模式下的定时器中断
    let sie = read_csr!("sie");
    let new_sie = insert_field(sie, SIE_STIE as u32, 0);
    write_csr!("sie",new_sie);
    
    reset_timer();
    increment_jiffies();
    println!("Core0 Timer interrupt received, jiffies={}",get_jiffies())

}

pub fn timer_init(){
    let sstatus = read_csr!("sstatus");
    let new_status = insert_field(sstatus, SSTATUS_SIE, 1);
    write_csr!("sstatus",new_status);
    reset_timer();
}
fn reset_timer() {
    sbi_set_timer(get_cycles() + CLINT_TIMEBASE_FREQ/HZ);
    // let sie = read_csr!("sie");
    // let new_sie = insert_field(sie, SIE_STIE as u32, 1);
    // write_csr!("sie",new_sie);
}
fn get_cycles() -> usize {
    unsafe { readq(VIRT_CLINT_TIMER_VAL as *const usize) }
}
#[inline(always)]
pub unsafe fn readq(addr: *const usize) -> usize {
    let value = read_volatile(addr);
    fence();
    value
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