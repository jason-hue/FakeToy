use crate::{print, println, read_csr};
use core::arch::asm;
use core::ptr;
use crate::trap::trap_context::TrapContext;
use crate::uart::{uart_init, uart_send};

pub const CAUSE_MISALIGNED_FETCH: usize = 0x0;
pub const CAUSE_FETCH_ACCESS: usize = 0x1;
pub const CAUSE_ILLEGAL_INSTRUCTION: usize = 0x2;
pub const CAUSE_BREAKPOINT: usize = 0x3;
pub const CAUSE_MISALIGNED_LOAD: usize = 0x4;
pub const CAUSE_LOAD_ACCESS: usize = 0x5;
pub const CAUSE_MISALIGNED_STORE: usize = 0x6;
pub const CAUSE_STORE_ACCESS: usize = 0x7;
pub const CAUSE_USER_ECALL: usize = 0x8;
pub const CAUSE_SUPERVISOR_ECALL: usize = 0x9;
pub const CAUSE_VIRTUAL_SUPERVISOR_ECALL: usize = 0xa;
pub const CAUSE_MACHINE_ECALL: usize = 0xb;
pub const CAUSE_FETCH_PAGE_FAULT: usize = 0xc;
pub const CAUSE_LOAD_PAGE_FAULT: usize = 0xd;
pub const CAUSE_STORE_PAGE_FAULT: usize = 0xf;
pub const CAUSE_FETCH_GUEST_PAGE_FAULT: usize = 0x14;
pub const CAUSE_LOAD_GUEST_PAGE_FAULT: usize = 0x15;
pub const CAUSE_VIRTUAL_INST_FAULT: usize = 0x16;
pub const CAUSE_STORE_GUEST_PAGE_FAULT: usize = 0x17;

pub const SBI_SET_TIMER: usize = 0;
pub const SBI_CONSOLE_PUTCHAR: usize = 1;
pub const SBI_CONSOLE_GETCHAR: usize = 2;

#[no_mangle]
pub fn sbi_trap_handler(regs: *mut TrapContext) -> isize{
    let trap_context = unsafe { &mut *regs };
    let mcause = read_csr!("mcause") as usize;
    let syscall_id = trap_context.x[12];//这里不是x[17]哦，x[]存的是x5~x31
    let mut ret: isize = 0;
    match mcause {
        CAUSE_SUPERVISOR_ECALL => {
            ret = sbi_ecall_handle(syscall_id,trap_context);
        }
        //..........扩展更多具体异常处理函数
        _ => {
            println!("Unhandled trap: {:#x}", mcause);
            ret = -1;
        }
    }
    ret

}

fn sbi_ecall_handle(syscall_id: usize, trap_context: &mut TrapContext) -> isize {
    let mut ret = 0;
    match syscall_id {
        SBI_CONSOLE_PUTCHAR => {
            uart_send(trap_context.x[5] as u8 as char);
            ret = 0;
        }
        SBI_SET_TIMER => {
            println!("No imply SBI_SET_TIMER!");
            ret = -1;
        }
        SBI_CONSOLE_GETCHAR => {
            println!("No imply SBI_CONSOLE_GETCHAR!");
            ret = -1;
        }
        _ => {
            println!("Unhandled syscall: {:#x}", syscall_id);
            ret = -1;
        }
    }
    trap_context.mepc += 4;
    ret
}