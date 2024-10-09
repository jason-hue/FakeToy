#![allow(dead_code)]

use core::arch::asm;
use core::ptr::{read_volatile, write_volatile};

// Memory barrier
#[inline(always)]
pub fn fence() {
    unsafe {
        asm!("fence iorw, iorw");
    }
}
// Read a 32-bit value from a memory address
#[inline(always)]
pub unsafe fn readl(addr: *const u32) -> u32 {
    let value = read_volatile(addr);
    fence();
    value
}

// Write a 32-bit value to a memory address
#[inline(always)]
pub unsafe fn writel(value: u32, addr: *mut u32) {
    fence();
    write_volatile(addr, value);
}

// Read an 8-bit value from a memory address
#[inline(always)]
pub unsafe fn readb(addr: *const u8) -> u8 {
    let value = read_volatile(addr);
    fence();
    value
}

// Write an 8-bit value to a memory address
#[inline(always)]
pub unsafe fn writeb(value: u8, addr: *mut u8) {
    fence();
    write_volatile(addr, value);
}

// Simple delay function
#[inline(always)]
pub fn delay(mut n: u32) {
    while n != 0 {
        n -= 1;
    }
}