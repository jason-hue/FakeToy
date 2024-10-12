use core::arch::asm;
use paste::paste;
// 常量定义
pub const PRV_U: u32 = 0;
pub const PRV_S: u32 = 1;
pub const PRV_M: u32 = 3;

pub const SSTATUS_SPP_SHIFT: u32 = 9;
pub const SSTATUS_SPP: u32 = 1 << SSTATUS_SPP_SHIFT;
pub const SSTATUS_SPIE: u32 = 0x00000020;
///在 val 中插入 fieldval，插入的位置和大小由 which 决定。
#[inline]
pub fn insert_field(val: u32, which: u32, fieldval: u32) -> u32 {
    (val & !which) | (fieldval << which.trailing_zeros())
}
///不能在运行时动态获取 CSR 名称,使用宏可以使csr名称在编译时确定
// CSR 读取操作
#[macro_export]
macro_rules! read_csr {
    ($csr:expr) => {{
        let value: u32;
        unsafe {
            asm!(concat!("csrr {}, ", $csr), out(reg) value);
        }
        value
    }};
}

// CSR 写入操作
#[macro_export]
macro_rules! write_csr {
    ($csr:expr, $val:expr) => {
        unsafe {
            asm!(concat!("csrw ", $csr, ", {}"), in(reg) $val);
        }
    };
}

// 为常用的 CSR 提供具体的读写函数
macro_rules! define_csr_functions {
    ($csrname:ident) => {
        paste::paste! {
            #[inline]
            pub fn [<read_ $csrname>]() -> u32 {
                read_csr!(stringify!($csrname))
            }

            #[inline]
            pub fn [<write_ $csrname>](val: u32) {
                write_csr!(stringify!($csrname), val)
            }
        }
    };
}

// 定义常用 CSR 的读写函数
define_csr_functions!(sstatus);
define_csr_functions!(sepc);
define_csr_functions!(stval);
define_csr_functions!(scause);
define_csr_functions!(stvec);
define_csr_functions!(sie);
define_csr_functions!(sip);