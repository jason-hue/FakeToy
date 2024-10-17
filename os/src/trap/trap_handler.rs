use core::fmt;
use crate::println;
use crate::trap::trap_context::TrapContext;
const SCAUSE_INT: usize = 1 << 63;
const SCAUSE_CODE_MASK: usize = 0x7fffffffffffffff;
// 定义一个异常处理函数的类型
type FaultHandler = fn(*mut TrapContext);
// 定义异常信息结构
struct FaultInfo {
    handler: FaultHandler,
    name: &'static str,
}

impl fmt::Debug for FaultInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FaultInfo {{ handler: {:p}, name: {} }}", self.handler as *const (), self.name)
    }
}

// 定义各种异常处理函数
fn do_trap_insn_misaligned(regs: *mut TrapContext) {
    let trap_context = unsafe { &mut *regs };
    unsafe {
        println!("Instruction address misaligned at sepc: 0x{:x}", trap_context.sepc);
        (*regs).sepc += 4;
        panic!();
    }
}

fn do_trap_insn_fault(regs: *mut TrapContext) {
    unsafe {
        println!("Instruction access fault at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
        panic!();
    }
}

fn do_trap_insn_illegal(regs: *mut TrapContext) {
    unsafe {
        println!("Illegal instruction at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
    }
}

fn do_trap_break(regs: *mut TrapContext) {
    unsafe {
        println!("Breakpoint at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
        panic!();
    }
}

fn do_trap_load_misaligned(regs: *mut TrapContext) {
    unsafe {
        println!("Load address misaligned at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
        panic!();
    }
}

fn do_trap_load_fault(regs: *mut TrapContext) {
    unsafe {
        println!("Load access fault at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
        panic!();
    }
}

fn do_trap_store_misaligned(regs: *mut TrapContext) {
    unsafe {
        println!("Store/AMO address misaligned at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
    }
}

fn do_trap_store_fault(regs: *mut TrapContext) {
    unsafe {
        println!("Store/AMO access fault at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
    }
}

fn do_trap_ecall_u(regs: *mut TrapContext) {
    unsafe {
        println!("Environment call from U-mode at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
    }
}

fn do_trap_ecall_s(regs: *mut TrapContext) {
    unsafe {
        println!("Environment call from S-mode at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
    }
}

fn do_trap_unknown(regs: *mut TrapContext) {
    unsafe {
        println!("Unknown exception at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
    }
}

fn do_page_fault(regs: *mut TrapContext) {
    unsafe {
        println!("Page fault at sepc: 0x{:x}", (*regs).sepc);
        (*regs).sepc = ((*regs).sepc & !1) + 4;
    }
}

// 定义静态异常信息数组
static FAULT_INFO: [FaultInfo; 16] = [
    FaultInfo { handler: do_trap_insn_misaligned, name: "Instruction address misaligned" },
    FaultInfo { handler: do_trap_insn_fault, name: "Instruction access fault" },
    FaultInfo { handler: do_trap_insn_illegal, name: "Illegal instruction" },
    FaultInfo { handler: do_trap_break, name: "Breakpoint" },
    FaultInfo { handler: do_trap_load_misaligned, name: "Load address misaligned" },
    FaultInfo { handler: do_trap_load_fault, name: "Load access fault" },
    FaultInfo { handler: do_trap_store_misaligned, name: "Store/AMO address misaligned" },
    FaultInfo { handler: do_trap_store_fault, name: "Store/AMO access fault" },
    FaultInfo { handler: do_trap_ecall_u, name: "Environment call from U-mode" },
    FaultInfo { handler: do_trap_ecall_s, name: "Environment call from S-mode" },
    FaultInfo { handler: do_trap_unknown, name: "unknown 10" },
    FaultInfo { handler: do_trap_unknown, name: "unknown 11" },
    FaultInfo { handler: do_page_fault, name: "Instruction page fault" },
    FaultInfo { handler: do_page_fault, name: "Load page fault" },
    FaultInfo { handler: do_trap_unknown, name: "unknown 14" },
    FaultInfo { handler: do_page_fault, name: "Store/AMO page fault" },
];

// 异常处理函数
#[link_section = ".text"]
#[no_mangle]
pub fn do_exception(regs: *mut TrapContext,scause: *mut usize) {
    println!("scause:0x{:016x}",scause as usize);
    if is_interrupt_fault(scause as usize){
        //处理中断
    }else {
        //处理异常
        let exception_code = get_exception_code(scause as usize);
        if exception_code < FAULT_INFO.len() {
            let fault_info = &FAULT_INFO[exception_code];
            println!("Handling exception: {}", fault_info.name);
            (fault_info.handler)(regs);
        } else {
            println!("Unknown exception code: {}", exception_code);
        }
    }
}
pub fn is_interrupt_fault(scause: usize) -> bool {
    scause & SCAUSE_INT != 0
}
fn get_exception_code(scause: usize) -> usize {
    scause & SCAUSE_CODE_MASK
}