const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_SHUTDOWN: usize = 8;
use core::arch::asm;

pub fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall ",
            inlateout("a0") arg0 => ret,
            in("a1") arg1,
            in("a2") arg2,
            in("a7") which,
        );
    }
    ret
}
pub fn putchar(c: char) -> isize{
    let ret = sbi_call(SBI_CONSOLE_PUTCHAR, c as usize, 0, 0);
    ret
}
pub fn shutdown(){
    sbi_call(SBI_SHUTDOWN,0,0,0,);
}