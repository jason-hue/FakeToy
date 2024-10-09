#![allow(dead_code)]

use core::ops::RangeInclusive;
use crate::config::{UART16550_CLOCK, UART_DEFAULT_BAUD};
use crate::uart::io::{readb, writeb};

// Base address for UART
pub const UART: usize = 0x10000000;

// Register offsets
pub const UART_DAT: usize = UART + 0x00; // 数据寄存器
pub const UART_IER: usize = UART + 0x01; // 中断使能寄存器
pub const UART_IIR: usize = UART + 0x02; // 中断标识寄存器 (read only)
pub const UART_FCR: usize = UART + 0x02; // FIFO控制寄存器 (write only)
pub const UART_LCR: usize = UART + 0x03; // 线路控制寄存器
pub const UART_MCR: usize = UART + 0x04; // MODEN控制寄存器
pub const UART_LSR: usize = UART + 0x05; // 线路状态寄存器
pub const UART_MSR: usize = UART + 0x06; // MODEN状态寄存器

pub const UART_DLL: usize = UART + 0x00; // 预分频寄存器低8位
pub const UART_DLM: usize = UART + 0x01; // 预分频寄存器高8位
/*
UART_DAT (UART + 0x00)
用途：数据寄存器，用于发送和接收数据。
读取时：获取接收到的数据。
写入时：存储要发送的数据。


UART_DLL (UART + 0x00)
用途：波特率除数锁存器（低字节）。
当 LCR 的 DLAB 位设置时可访问。
与 DLM 一起用于设置波特率。


UART_DLM (UART + 0x01)
用途：波特率除数锁存器（高字节）。
当 LCR 的 DLAB 位设置时可访问。

UART_IER (UART + 0x01)
用途：中断使能寄存器。
控制哪些 UART 条件可以产生中断。


UART_FCR (UART + 0x02)
用途：FIFO 控制寄存器（只写）。
用于启用 FIFO、清除 FIFO、设置 FIFO 触发级别等。


UART_LCR (UART + 0x03)
用途：线路控制寄存器。
控制数据位数、停止位、奇偶校验、DLAB 等。


UART_MCR (UART + 0x04)
用途：调制解调器控制寄存器。
控制 RTS、DTR 等信号，以及回环模式。

UART_IIR (UART + 0x02)
用途：中断识别寄存器（只读）。
指示当前最高优先级的中断源。


UART_LSR (UART + 0x05)
用途：线路状态寄存器。
提供有关数据传输的状态信息。


UART_MSR (UART + 0x06)
用途：调制解调器状态寄存器。
提供 CTS、DSR、RI、DCD 等信号的状态。
*/

// LSR bit flags
pub struct UartLsr;

impl UartLsr {
    pub const ERROR: u8 = 0x80; // 出错
    pub const EMPTY: u8 = 0x40; // 传输FIFO和移位寄存器为空
    pub const TFE: u8 = 0x20;   // 传输FIFO为空
    pub const BI: u8 = 0x10;    // 传输被打断
    pub const FE: u8 = 0x08;    // 接收到没有停止位的帧
    pub const PE: u8 = 0x04;    // 奇偶校验错误位
    pub const OE: u8 = 0x02;    // 数据溢出
    pub const DR: u8 = 0x01;    // FIFO有数据
}

// 定义UART寄存器范围
pub const UART_REGISTER_RANGE: RangeInclusive<usize> = UART..=UART_MSR;

// 安全的读写UART寄存器的函数
pub unsafe fn read_uart_register(address: usize) -> u8 {
    if UART_REGISTER_RANGE.contains(&address) {
        (address as *const u8).read_volatile()
    } else {
        panic!("Attempt to read from invalid UART register address")
    }
}

pub unsafe fn write_uart_register(address: usize, value: u8) {
    if UART_REGISTER_RANGE.contains(&address) {
        (address as *mut u8).write_volatile(value);
    } else {
        panic!("Attempt to write to invalid UART register address")
    }
}

//发送一个字节
pub fn uart_send(c: char){
    unsafe {
        while readb(UART_LSR as *const u8) & UartLsr::EMPTY == 0 {}
        writeb(c as u8, UART_DAT as *mut u8);
    }
}

//发送字符串
pub fn uart_send_string(s: &str){
    for c in s.chars(){
        unsafe { uart_send(c); }
    }
}
pub fn uart_init(){
    let divisor = UART16550_CLOCK / (16*UART_DEFAULT_BAUD);
    unsafe {
        writeb(0, UART_IER as *mut u8); //关闭中断

        //使能DLAB(设置波特率除数)
        writeb(0x80,UART_LCR as *mut u8);
        writeb(divisor as u8,UART_DLL as *mut u8);
        writeb((divisor >> 8) as u8,UART_DLM as *mut u8);

        //8bit 无奇偶效验 停止位为1
        writeb(0x3,UART_LCR as *mut u8);

        //使能FIFO,清空FIFO，设置14字节threshold
        writeb(0xc7,UART_FCR as *mut u8);
    }
}