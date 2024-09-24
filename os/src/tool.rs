use core::arch::asm;
use core::ops::Add;
use core::ptr;
use crate::println;
use crate::config::{SBI_SYSTEM_RESET,SHUTDOWN_TYPE,SHUTDOWN_REASON};
pub unsafe  fn memcopy(mut src: *const u8, mut dst: *mut u8, mut size: usize){
    while (dst as usize)%8!=0 && size>0 {
        asm!(
        "lb {tmp},0({src})",
        "sb {tmp},0({dst})",
        src = in(reg) src,
        dst = in(reg) dst,
        tmp = out(reg) _
        );
        src = src.add(1);
        dst = dst.add(1);
        size -= 1;
    }//内存对齐
    while size >= 8 {
        asm!(
        "ld {tmp},0({src})",
        "sd {tmp},0({dst})",
        src = in(reg) src,
        dst = in(reg) dst,
        tmp = out(reg) _
        );
        src = src.add(8);
        dst = dst.add(8);
        size -= 8;
    }
    if size <= 8 {
        let mut value: usize = 0;
        asm!(
        "ld {value},0({src})",
        src = in(reg) src,
        value = out(reg) value
        );
        asm!(
        "sd {value},0({dst})",
        dst = in(reg) dst,
        value = in(reg) value
        );
        return;

        // let mut value: u64 = 0;
        // ptr::copy_nonoverlapping(src, &mut value as *mut u64 as *mut u8, size);
        // ptr::copy_nonoverlapping(&value as *const u64 as *const u8, dst, size);
        // return;
    }
    while size > 0 {
        asm!(
        "lb {tmp},0({src})",
        "sb {tmp},0({dst})",
        src = in(reg) src,
        dst = in(reg) dst,
        tmp = out(reg) _
        );
        src = src.add(1);
        dst = dst.add(1);
        size -= 1;
    }
}
pub fn memcopy_test(){
    let mut src1 = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut dst1 = [0u8; 10];
    let mut src2 = [1u8, 2, 3, 4];
    let mut dst2 = [0u8; 4];
    unsafe {
        memcopy(src1.as_ptr(), dst1.as_mut_ptr(), src1.len());
        memcopy(src2.as_ptr(), dst2.as_mut_ptr(), src2.len());
    }

    assert_eq!(src1, dst1);
    assert_eq!(src2, dst2);
    println!("[kernel]Memory copy test successful!");
}