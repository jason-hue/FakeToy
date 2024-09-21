use core::arch::asm;
use paste::paste;
// 常量定义
pub const PRV_U: usize = 0;
pub const PRV_S: usize = 1;
pub const PRV_M: usize = 3;

pub const MSTATUS_MPP_SHIFT: usize = 11;
pub const MSTATUS_MPP: usize = 3 << MSTATUS_MPP_SHIFT;
pub const MSTATUS_MPIE: usize = 0x00000080;
/*
例子：
let mstatus = read_crs(mstatus);
new mstatus = insert_filed(mstatus, MSTATUS_MPP, PRV_S);
write_csr(mstatus, new_mstatus);
这段代码会：
读取当前的 mstatus 值
在 mstatus 中插入新的 MPP 值（设置为 Supervisor 模式）
将更新后的值写回 mstatus CSR
 */

///在 val 中插入 fieldval，插入的位置和大小由 which 决定。
#[inline]
pub fn insert_field(val: usize, which: usize, fieldval: usize) -> usize {
    (val & !which) | (fieldval << which.trailing_zeros())
}
///不能在运行时动态获取 CSR 名称,使用宏可以使csr名称在编译时确定
// CSR 读取操作
#[macro_export]
macro_rules! read_csr {
    ($csr:expr) => {{
        let value: usize;
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
            pub fn [<read_ $csrname>]() -> usize {
                read_csr!(stringify!($csrname))
            }

            #[inline]
            pub fn [<write_ $csrname>](val: usize) {
                write_csr!(stringify!($csrname), val)
            }
        }
    };
}

// 定义常用 CSR 的读写函数
define_csr_functions!(mstatus);
define_csr_functions!(mepc);
define_csr_functions!(mtval);
define_csr_functions!(mcause);
define_csr_functions!(mtvec);
define_csr_functions!(mie);
define_csr_functions!(mip);