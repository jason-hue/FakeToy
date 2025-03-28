pub const KERNEL_HEAP_SIZE: usize = 0x100_0000;
pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_SIZE_BITS: usize = 0xc;
pub const MEMORY_END: usize = 0x8800_0000;
pub const SIE_SSIE: usize = 0x2;      // 1 << 1
pub const SIE_STIE: usize = 0x20;     // 1 << 5
pub const SIE_SEIE: usize = 0x200;    // 1 << 9
pub const VIRT_CLINT_ADDR: usize = 0x2000000;
pub const VIRT_CLINT_TIMER_CMP: usize = VIRT_CLINT_ADDR + 0x4000;
pub const VIRT_CLINT_TIMER_VAL: usize = VIRT_CLINT_ADDR + 0xbff8;
/// CLINT (Core Local Interruptor) 时基频率，10MHz
pub const CLINT_TIMEBASE_FREQ: usize = 10_000_000;

/// 系统时钟频率，1KHz
pub const HZ: usize = 1;
pub const MMIO: &[(usize, usize)] = &[
    (0x0010_0000, 0x00_2000), // VIRT_TEST/RTC  in virt machine
    (0x2000000, 0x10000),     // core local interrupter (CLINT)
    (0xc000000, 0x210000),    // VIRT_PLIC in virt machine
    (0x10000000, 0x9000),     // VIRT_UART0 with GPU  in virt machine
];