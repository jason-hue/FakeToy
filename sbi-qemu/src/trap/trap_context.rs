#[derive(Copy, Clone)]
#[repr(C)]
pub struct TrapContext {
    pub mepc: usize,
    pub x1: usize,
    pub sp:usize,//栈底指针
    pub x3: usize,
    pub ret: usize,//实际上x4 tp线程寄存器并没有保存，内存中这个位置用来保存返回值
    pub x: [usize;27],//x5~x31
    pub mstatus: usize,
}
// impl TrapContext {
//     pub fn new(sp: *mut usize) -> &'static mut Self {
//         unsafe { &mut *(sp as *mut TrapContext) }
//     }
// }
