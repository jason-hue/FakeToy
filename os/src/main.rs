#![feature(panic_info_message)]
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
extern crate alloc;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::csr::{read_sepc, read_sstatus};
use crate::mm::{frame_allocator_alloc_more_test, frame_allocator_test, heap_test};
use crate::sbi::{putchar, sbi_call, shutdown};
use crate::trap::trap_init;

mod config;
mod console;
mod panic_handler;
mod csr;
mod sbi;
mod trap;
mod mm;
global_asm!(include_str!("boot.S"));
global_asm!(include_str!("trap/trap.S"));
extern "C"{
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn stacks_start();
    fn stacks_end();
    fn start_with_stack();
    fn sbss();
    fn ebss();
}
fn clear_bss(){
    unsafe { (sbss as usize..ebss as usize).for_each(|a|{
        ( a as *mut u8).write_volatile(0)
    }) }
}
fn print_segment_info(){
    // 打印所有段的信息
    println!(".text section:   {:#x} - {:#x}", stext as usize, etext as usize);
    println!(".rodata section: {:#x} - {:#x}", srodata as usize, erodata as usize);
    println!("stack :          {:#x} - {:#x}", stacks_start as usize,stacks_end as usize);
    println!(".data section:   {:#x} - {:#x}", sdata as usize, edata as usize);
    println!(".bss section:    {:#x} - {:#x}", sbss as usize, ebss as usize);

    // 计算并打印每个段的大小
    println!(".text size:   {} bytes", etext as usize - stext as usize);
    // println!("stack size:   {} bytes", sdata as usize - start_with_stack as usize);
    println!("stack size:   {} bytes", stacks_end as usize - stacks_start as usize);
    println!(".rodata size: {} bytes", erodata as usize - srodata as usize);
    println!(".data size:   {} bytes", edata as usize - sdata as usize);
    println!(".bss size:    {} bytes", ebss as usize - sbss as usize);

    // 计算并打印整个内核的大小
    println!("Total kernel size: {} bytes", ebss as usize - stext as usize);

}
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    clear_bss();
    trap_init();
    mm::mm_init();
    println!("Entering kernel_main");
    print_segment_info();
    heap_test();
    frame_allocator_alloc_more_test();
    frame_allocator_test();
    //trap test
    unsafe {
        asm!("mret");
        asm!("csrrw sp,mscratch,sp")
    }
    println!("System running normally");
    println!("Shutdown!");
    shutdown();
    unreachable!()
}