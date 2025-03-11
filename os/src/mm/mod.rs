mod page_table;
mod heap_allocator;
mod address;
mod frame_allocator;
mod address_space;
pub use heap_allocator::heap_test;
pub use frame_allocator::{frame_allocator_alloc_more_test,frame_allocator_test};
pub use address_space::{AddressSpace, MapRegion, MapType, MapPermission};
pub use address::VirtAddr;
use core::arch::asm;
use crate::config::MEMORY_END;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::println;

lazy_static! {
    pub static ref KERNEL_SPACE: Mutex<AddressSpace> = Mutex::new(AddressSpace::new());
}

extern "C" {
    fn stext();
    fn etext();
    fn srodata();
    fn erodata();
    fn sdata();
    fn edata();
    fn sbss();
    fn ebss();
    fn stacks_start();
    fn stacks_end();
    fn ekernel();
}

pub fn mm_init(){
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    
    println!("Start mapping kernel sections...");
    
    // 映射内核代码段(.text)
    let text_region = MapRegion::new(
        VirtAddr::from(stext as usize),
        VirtAddr::from(etext as usize),
        MapType::Identity,
        MapPermission::R | MapPermission::X,
    );
    KERNEL_SPACE.lock().insert_region(text_region);
    println!(".text section mapped: {:#x} - {:#x}", stext as usize, etext as usize);
    
    // 映射内核只读数据段(.rodata)
    let rodata_region = MapRegion::new(
        VirtAddr::from(srodata as usize),
        VirtAddr::from(erodata as usize),
        MapType::Identity,
        MapPermission::R,
    );
    KERNEL_SPACE.lock().insert_region(rodata_region);
    println!(".rodata section mapped: {:#x} - {:#x}", srodata as usize, erodata as usize);
    
    // 映射内核数据段(.data)
    let data_region = MapRegion::new(
        VirtAddr::from(sdata as usize),
        VirtAddr::from(edata as usize),
        MapType::Identity,
        MapPermission::R | MapPermission::W,
    );
    KERNEL_SPACE.lock().insert_region(data_region);
    println!(".data section mapped: {:#x} - {:#x}", sdata as usize, edata as usize);
    
    // 映射内核BSS段(.bss)
    let bss_region = MapRegion::new(
        VirtAddr::from(sbss as usize),
        VirtAddr::from(ebss as usize),
        MapType::Identity,
        MapPermission::R | MapPermission::W,
    );
    KERNEL_SPACE.lock().insert_region(bss_region);
    println!(".bss section mapped: {:#x} - {:#x}", sbss as usize, ebss as usize);
    
    // 映射内核栈
    let stack_region = MapRegion::new(
        VirtAddr::from(stacks_start as usize),
        VirtAddr::from(stacks_end as usize),
        MapType::Identity,
        MapPermission::R | MapPermission::W,
    );
    KERNEL_SPACE.lock().insert_region(stack_region);
    println!("Kernel stack mapped: {:#x} - {:#x}", stacks_start as usize, stacks_end as usize);
    
    // 映射全部物理内存（用于访问任意物理地址）
    let memory_region = MapRegion::new(
        VirtAddr::from(ekernel as usize),
        VirtAddr::from(MEMORY_END),
        MapType::Identity,
        MapPermission::R | MapPermission::W | MapPermission::X,
    );
    KERNEL_SPACE.lock().insert_region(memory_region);
    println!("Physical memory mapped: {:#x} - {:#x}", 0x8000_0000usize, MEMORY_END);
    
    println!("About to enable paging...");
    
    // 启用分页
    unsafe {
        // 设置satp寄存器并刷新TLB
        let satp = KERNEL_SPACE.lock().token();
        println!("Setting satp = {:#x}", satp);
        
        // 写入satp寄存器并刷新TLB
        asm!("csrw satp, {}", in(reg) satp);
        asm!("sfence.vma");
        
        // 验证分页是否生效
        println!("Paging enabled successfully");
    }
    
    // 测试分页功能
    test_paging();
}

// 测试分页功能是否正确
pub fn test_paging() {
    println!("Testing paging functionality...");
    
    // 测试1: 验证虚拟地址转换
    let va = VirtAddr::from(0x80200000); // 内核代码段起始地址
    let pa = KERNEL_SPACE.lock().translate(va);
    match pa {
        Some(pa) => println!("Virtual address 0x{:#x} maps to physical address 0x{:#x}", va.0, pa.0),
        None => println!("Failed to translate virtual address 0x{:#x}", va.0),
    }
    
    // 测试2: 尝试访问已映射的内存
    unsafe {
        let ptr = 0x80200000 as *const u8;
        let value = ptr.read_volatile();
        println!("Value at 0x80200000: 0x{:#x}", value);
    }
    
    // 测试3: 验证satp寄存器是否设置正确
    let mut satp_value: usize;
    unsafe {
        asm!("csrr {}, satp", out(reg) satp_value);
    }
    println!("Current satp value: 0x{:#x}", satp_value);
    
    // 测试4: 验证TLB刷新是否有效
    unsafe {
        asm!("sfence.vma");
        println!("TLB flushed successfully");
    }
    
    println!("Paging test completed");
}