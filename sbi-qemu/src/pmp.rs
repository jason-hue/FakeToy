use core::arch::asm;
use crate::println;

#[derive(Clone, Copy)]
struct PmpConfig {
    addr: usize,
    size: usize,
    prot: u8,
}

const PMP_R: u8 = 1 << 0;
const PMP_W: u8 = 1 << 1;
const PMP_X: u8 = 1 << 2;

unsafe fn set_pmp(entry: usize, cfg: PmpConfig) {
    let addr = if cfg.size == usize::MAX / 2 + 1 {
        cfg.addr >> 2 // TOR mode for the first entry
    } else {
        assert!(cfg.size.is_power_of_two(), "PMP size must be a power of two");
        (cfg.addr | (cfg.size / 2 - 1)) >> 2 // NAPOT mode
    };
    match entry {
        0 => asm!("csrw pmpaddr0, {}", in(reg) addr),
        1 => asm!("csrw pmpaddr1, {}", in(reg) addr),
        2 => asm!("csrw pmpaddr2, {}", in(reg) addr),
        3 => asm!("csrw pmpaddr3, {}", in(reg) addr),
        _ => panic!("Unsupported PMP entry: {}", entry),
    }
    let pmpcfg = if cfg.size == usize::MAX / 2 + 1 {
        cfg.prot & 0x7 | 0x08 // TOR 模式
    } else {
        cfg.prot & 0x7 | 0x18 // NAPOT 模式
    };
    let shift = (entry % 8) * 8;
    let pmpcfg_val = (pmpcfg as u64) << shift;

    // 根据 entry 选择正确的 pmpcfg 寄存器
    if entry < 8 {
        asm!(
        "csrr {tmp}, pmpcfg0",
        "and {tmp}, {tmp}, {mask}",
        "or {tmp}, {tmp}, {val}",
        "csrw pmpcfg0, {tmp}",
        tmp = out(reg) _,
        mask = in(reg) !(0xffu64 << shift),
        val = in(reg) pmpcfg_val,
        options(nomem, nostack)
        );
    } else {
        asm!(
        "csrr {tmp}, pmpcfg2",
        "and {tmp}, {tmp}, {mask}",
        "or {tmp}, {tmp}, {val}",
        "csrw pmpcfg2, {tmp}",
        tmp = out(reg) _,
        mask = in(reg) !(0xffu64 << shift),
        val = in(reg) pmpcfg_val,
        options(nomem, nostack)
        );
    }
}

pub fn init_pmp() {
    unsafe {
        // pmp01: 0x00000000..0x80000000 (-wr)
        set_pmp(0, PmpConfig {
            addr: 0x00000000,
            size: 0x80000000,
            prot: PMP_R | PMP_W,
        });

        // pmp02: 0x80000000..0x80200000 (---)
        set_pmp(1, PmpConfig {
            addr: 0x80000000,
            size: 0x00200000,
            prot: 0,
        });

        // pmp03: 0x80200000..0x88000000 (xwr)
        set_pmp(2, PmpConfig {
            addr: 0x80200000,
            size: 0x08000000,
            prot: PMP_X | PMP_W | PMP_R,
        });

        // pmp04: 0x88000000..0xFFFFFFFF (-wr)
        set_pmp(3, PmpConfig {
            addr: 0x88000000,
            size: usize::MAX / 2 + 1,
            prot: PMP_R | PMP_W,
        });
    }
}

// 打印PMP配置的函数
fn print_pmp_config(entry: usize, start: usize, end: usize, prot: u8) {
    let prot_str = match prot {
        0 => "---",
        1 => "r--",
        2 => "-w-",
        3 => "rw-",
        4 => "--x",
        5 => "r-x",
        6 => "-wx",
        7 => "rwx",
        _ => "???",
    };
    println!("[sbi] pmp{:02}: 0x{:08x}..0x{:08x} ({})", entry + 1, start, end, prot_str);
}

pub fn print_pmp_info() {
    print_pmp_config(0, 0x00000000, 0x80000000, PMP_R | PMP_W);
    print_pmp_config(1, 0x80000000, 0x80200000, 0);
    print_pmp_config(2, 0x80200000, 0x88000000, PMP_X | PMP_W | PMP_R);
    print_pmp_config(3, 0x88000000, 0x00000000, PMP_R | PMP_W);
}