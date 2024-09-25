use core::arch::asm;
use core::ops::Add;
use crate::println;
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
pub unsafe fn memset(mut src: *mut u8, value: u8, mut count: u64){
    let value_64 = (value as u64) * 0x0101010101010101u64;

    // 处理未对齐的字节
    while (src as usize) % 8 != 0 && count > 0 {
        asm!(
        "sb {value}, 0({src})",
        src = in(reg) src,
        value = in(reg) value,
        );
        src = src.add(1);
        count -= 1;
    }

    // 使用 64 位操作设置 8 字节对齐的部分
    while count >= 8 {
        asm!(
        "sd {value}, 0({src})",
        value = in(reg) value_64,
        src = in(reg) src,
        );
        src = src.add(8);
        count -= 8;
    }

    // 处理剩余的字节
    while count > 0 {
        asm!(
        "sb {value}, 0({src})",
        src = in(reg) src,
        value = in(reg) value,
        );
        src = src.add(1);
        count -= 1;
    }
}
pub fn combined_memory_test() {
    println!("[DEBUG] Starting combined memory test");

    // memcopy test
    let src1 = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut dst1 = [0u8; 10];
    unsafe {
        memcopy(src1.as_ptr(), dst1.as_mut_ptr(), src1.len());
    }
    println!("[DEBUG] After memcopy: dst1 = {:?}", dst1);
    assert_eq!(src1, dst1, "Memcopy failed in combined test");

    // memset test
    let mut src2 = [0u8; 10];
    let value = 5u8;
    unsafe {
        memset(src2.as_mut_ptr(), value, src2.len() as u64);
    }
    println!("[DEBUG] After memset: src2 = {:?}", src2);
    assert_eq!(src2, [5u8; 10], "Memset failed in combined test");

    println!("[kernel] Combined memory test successful!");
}