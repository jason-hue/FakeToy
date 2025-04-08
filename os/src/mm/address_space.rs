use alloc::vec::Vec;
use bitflags::bitflags;
use crate::mm::address::{PhysAddr, PhysPageNum, VPNRange, VirtAddr, VirtPageNum};
use crate::mm::frame_allocator::{frame_alloc, FrameTracker};
use crate::mm::page_table::{PageTable, PageTableEntry, PTEFlags};
use alloc::collections::BTreeMap;
use crate::config::PAGE_SIZE;
use crate::{print, println};

// 抽象地址空间结构
pub struct AddressSpace {
    // 页表
    page_table: PageTable,
    // 地址空间中的映射区域
    regions: Vec<MapRegion>,
}

impl AddressSpace {
    // 创建一个新的地址空间
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            regions: Vec::new(),
        }
    }

    // 从现有的token创建地址空间
    pub fn from_token(token: usize) -> Self {
        Self {
            page_table: PageTable::from_token(token),
            regions: Vec::new(),
        }
    }

    // 获取页表token
    pub fn token(&self) -> usize {
        self.page_table.token()
    }

    // 添加一个新的映射区域
    pub fn insert_region(&mut self, mut region: MapRegion) {
        println!("[MAPPING] Adding new region: VPN range: {:?}, type: {:?}", 
            region.vpn_range, region.map_type);
        region.map_region(&mut self.page_table);
        self.regions.push(region);
        // 打印页表信息
        println!("[MAPPING] Region mapping completed");
        self.debug_map_info();
    }

    // 移除一个映射区域
    pub fn remove_region(&mut self, index: usize) {
        if let Some(region) = self.regions.get_mut(index) {
            region.unmap_region(&mut self.page_table);
            self.regions.remove(index);
        }
    }

    // 将一个虚拟地址转换为物理地址
    pub fn translate(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.page_table.translate_va_pa(va)
    }

    // 打印页表项信息，用于调试
    pub fn print_pagetable(&self) {
        println!("[PageTable Debug Info]");
        println!("PageTable Token: {:#x}", self.token());
        println!("MapArea Num: {}", self.regions.len());
        
        for (i, region) in self.regions.iter().enumerate() {
            println!("Area #{}: VPN range: {:?}, map type: {:?}",
                i, region.vpn_range, region.map_type);
            println!("permissions: {:?}", region.permission);
            
            // 打印该区域中的一些页表项
            let mut count = 0;
            for vpn in region.vpn_range {
                if let Some(pte) = self.page_table.find_pte(vpn.into()) {
                    let va: VirtAddr = vpn.into();
                    println!("  VPN: {:#010x} -> PTE: {:#018x}, flag: {:?}",
                        va.0, pte.bits, pte.flags());
                } else {
                    let va: VirtAddr = vpn.into();
                    println!("  VPN: {:#010x} -> No Mapped", va.0);
                }
                
                count += 1;
                if count >= 5 {
                    println!("  ...(More entries omitted...)");
                    break;
                }
            }
        }
        println!("[End of PageTable Debug Info]");
    }

    // Low-level page table debugging function
    pub fn dump_page_table(&self) {
        println!("[PAGE_TABLE_DUMP]");
        println!("SATP: {:#x}", self.token());
        println!("ROOT_PPN: {:#x}", self.page_table.root_ppn.0);
        println!("MAPPED_REGIONS: {}", self.regions.len());
        
        for (i, region) in self.regions.iter().enumerate() {
            println!("REGION[{}]: START_VPN={:#010x} END_VPN={:#010x} TYPE={:?}", 
                i, 
                (region.vpn_range.get_start().0 << 12), 
                (region.vpn_range.get_end().0 << 12) - 1, 
                region.map_type);
            println!("FLAGS: V={} R={} W={} X={} U={}", 
                (region.permission & PTEFlags::V) != PTEFlags::empty(),
                (region.permission & PTEFlags::R) != PTEFlags::empty(),
                (region.permission & PTEFlags::W) != PTEFlags::empty(),
                (region.permission & PTEFlags::X) != PTEFlags::empty(),
                (region.permission & PTEFlags::U) != PTEFlags::empty());
            
            // Walk through the page table for this region and dump entries
            for vpn in region.vpn_range {
                if let Some(pte) = self.page_table.find_pte(vpn.into()) {
                    let flags = pte.flags();
                    let va: VirtAddr = vpn.into();
                    let pa: PhysAddr = pte.ppn().into();
                    println!("  VPN={:#010x} PPN={:#010x} V={} R={} W={} X={} U={} RAW={:#018x}", 
                        va.0, 
                        pa.0,
                        (flags & PTEFlags::V) != PTEFlags::empty(),
                        (flags & PTEFlags::R) != PTEFlags::empty(),
                        (flags & PTEFlags::W) != PTEFlags::empty(),
                        (flags & PTEFlags::X) != PTEFlags::empty(),
                        (flags & PTEFlags::U) != PTEFlags::empty(),
                        pte.bits);
                } else {
                    let va: VirtAddr = vpn.into();
                    println!("  VPN={:#010x} [UNMAPPED]", va.0);
                }
            }
        }
        
        // Dump raw page table structure (multi-level)
        println!("[HIERARCHICAL_PAGE_TABLE]");
        self.walk_page_table_recursive(self.page_table.root_ppn, 0, 0);
        println!("[END_PAGE_TABLE_DUMP]");
    }
    
    // Recursively walk through the page table to show its hierarchy
    fn walk_page_table_recursive(&self, ppn: PhysPageNum, level: usize, idx_prefix: usize) {
        if level >= 4 {
            return;
        }
        
        let pte_array = ppn.get_pte_array();
        let indent = "  ".repeat(level);
        
        println!("{}LEVEL_{}_TABLE @ PPN={:#010x}", indent, level, (ppn.0 << 12));
        
        // Print only valid entries to avoid excessive output
        for i in 0..512 {
            let pte = &pte_array[i];
            if pte.is_valid() {
                let next_ppn = pte.ppn();
                let idx = (idx_prefix << 9) | i;
                let pa: PhysAddr = next_ppn.into();
                println!("{}[{}] IDX={:#05x} PPN={:#010x} FLAGS={:#010b}", 
                    indent, i, idx, pa.0, pte.flags().bits());
                
                // Recursively walk the next level if this is not a leaf
                if level < 3 {
                    self.walk_page_table_recursive(next_ppn, level + 1, idx);
                }
            }
        }
    }

    // Advanced page table dump with hexdump
    pub fn dump_page_table_detailed(&self) {
        println!("[DETAILED_PAGE_TABLE_DUMP]");
        println!("SATP: {:#x} ROOT_PPN: {:#x}", self.token(), self.page_table.root_ppn.0);
        
        // Dump memory layout of each mapped page
        for (i, region) in self.regions.iter().enumerate() {
            println!("\nREGION[{}]: TYPE={:?} RANGE={:#x}-{:#x}", 
                i, region.map_type, 
                region.vpn_range.get_start().0 << 12, 
                (region.vpn_range.get_end().0 << 12) - 1);
            
            // Display memory content if Framed type (with physical memory)
            if region.map_type == MapType::Framed {
                let mut count = 0;
                for vpn in region.vpn_range {
                    if let Some(pte) = self.page_table.find_pte(vpn.into()) {
                        if pte.is_valid() {
                            let ppn = pte.ppn();
                            let va: VirtAddr = vpn.into();
                            println!("  VPN={:#x} MAPS_TO PPN={:#x} VA={:#x} PA={:#x}", 
                                vpn.0, ppn.0, va.0, usize::from(PhysAddr::from(ppn)));
                            
                            // Perform hexdump of page content (first 128 bytes)
                            Self::hexdump(ppn, 128);
                            
                            count += 1;
                            if count >= 2 {  // Limit to just 2 pages per region
                                println!("  ... more pages not shown ...");
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        // Dump complete PTE chains for a few sample addresses
        if !self.regions.is_empty() {
            println!("\n[PTE_CHAINS]");
            // Choose a few sample VPNs to trace through all levels
            let samples = self.get_sample_vpns(3);
            for vpn in samples {
                self.dump_pte_chain(vpn);
            }
        }
        
        println!("[END_DETAILED_PAGE_TABLE_DUMP]");
    }
    
    // Get a sample of VPNs from different regions
    fn get_sample_vpns(&self, count: usize) -> Vec<VirtPageNum> {
        let mut samples = Vec::new();
        for region in self.regions.iter() {
            if let Some(vpn) = region.vpn_range.into_iter().next() {
                samples.push(vpn);
                if samples.len() >= count {
                    break;
                }
            }
        }
        samples
    }
    
    // Dump the PTE chain for a specific VPN
    fn dump_pte_chain(&self, vpn: VirtPageNum) {
        println!("PTE_CHAIN FOR VPN={:#x}:", vpn.0);
        
        let idxs = vpn.indexes();
        let mut ppn = self.page_table.root_ppn;
        
        println!("  ROOT_PAGE @ PPN={:#x}", ppn.0);
        
        for (level, &idx) in idxs.iter().enumerate() {
            let pte_array = ppn.get_pte_array();
            let pte = &pte_array[idx];
            
            if !pte.is_valid() {
                println!("  L{}_PTE[{:#x}]: INVALID", level, idx);
                break;
            }
            
            println!("  L{}_PTE[{:#x}]: {:#x} -> PPN={:#x} FLAGS={:#b}", 
                level, idx, pte.bits, pte.ppn().0, pte.flags().bits());
            
            if level == 3 {
                // Final level PTE points to data
                println!("  DATA_PAGE @ PPN={:#x} PA={:#x}", 
                    pte.ppn().0, usize::from(PhysAddr::from(pte.ppn())));
            } else {
                // Continue to next level
                ppn = pte.ppn();
            }
        }
    }
    
    // Helper function to create hexdump of memory
    fn hexdump(ppn: PhysPageNum, bytes: usize) {
        let byte_array = ppn.get_bytes_array();
        let limit = bytes.min(PAGE_SIZE);
        
        println!("  [HEXDUMP] First {} bytes at PPN={:#x}:", limit, ppn.0);
        
        for row in 0..(limit + 15) / 16 {
            let start = row * 16;
            if start >= limit {
                break;
            }
            
            // Print address
            print!("  {:#06x}:", start);
            
            // Print hex values
            for col in 0..16 {
                let idx = start + col;
                if idx < limit {
                    print!(" {:02x}", byte_array[idx]);
                } else {
                    print!("   ");
                }
                
                // Extra space between groups of 8
                if col == 7 {
                    print!(" ");
                }
            }
            
            // Print ASCII representation
            print!("  |");
            for col in 0..16 {
                let idx = start + col;
                if idx < limit {
                    let c = byte_array[idx];
                    // Print dot for non-printable characters
                    if c >= 32 && c <= 126 {
                        print!("{}", c as char);
                    } else {
                        print!(".");
                    }
                } else {
                    print!(" ");
                }
            }
            println!("|");
        }
    }

    // Generate a raw page table view with bit-by-bit explanation
    pub fn raw_page_table_view(&self) {
        println!("[RAW_PAGE_TABLE_VIEW]");
        println!("SATP: {:#016x}", self.token());
        println!("  MODE: {}", (self.token() >> 60) & 0xF);
        println!("  ASID: {:#x}", (self.token() >> 44) & 0xFFFF);
        println!("  PPN:  {:#x}", self.token() & ((1 << 44) - 1));
        
        // Examine the first level page directory
        let root_ppn = self.page_table.root_ppn;
        println!("\nROOT PAGE DIRECTORY @ PPN={:#x}:", root_ppn.0);
        println!("Physical Address: {:#x}", usize::from(PhysAddr::from(root_ppn)));
        
        // Show bit field structure for PTEs
        println!("\nPAGE TABLE ENTRY BIT FIELDS:");
        println!("  63:10 PPN    - Physical Page Number");
        println!("  9:8   RSW    - Reserved for Supervisor Software");
        println!("  7     D      - Dirty");
        println!("  6     A      - Accessed");
        println!("  5     G      - Global Mapping");
        println!("  4     U      - User Mode Access");
        println!("  3     X      - Executable");
        println!("  2     W      - Writable");
        println!("  1     R      - Readable");
        println!("  0     V      - Valid");
        
        // Raw hexdump of first 16 PTEs from each level
        self.dump_raw_pte_level(root_ppn, 0);
        
        // Find one valid entry to follow through all levels
        self.follow_pte_chain_from_root();
        
        println!("[END_RAW_PAGE_TABLE_VIEW]");
    }
    
    // Dump raw PTE values at a specific level
    fn dump_raw_pte_level(&self, ppn: PhysPageNum, level: usize) {
        let pte_array = ppn.get_pte_array();
        
        println!("\nLEVEL {} PAGE TABLE RAW ENTRIES (First 16):", level);
        println!("IDX    PTE_RAW           FLAGS V R W X U G A D    PPN");
        println!("----------------------------------------------------------");
        
        let mut found_valid = false;
        let mut next_ppn = None;
        
        for i in 0..16 {
            let pte = &pte_array[i];
            let valid = pte.is_valid();
            let flags = pte.flags();
            
            println!("{:#04x}: {:#018x}   {:#04b} {} {} {} {} {} {} {} {}    {:#x}", 
                i,
                pte.bits,
                flags.bits(),
                if (flags & PTEFlags::V) != PTEFlags::empty() { "1" } else { "0" },
                if (flags & PTEFlags::R) != PTEFlags::empty() { "1" } else { "0" },
                if (flags & PTEFlags::W) != PTEFlags::empty() { "1" } else { "0" },
                if (flags & PTEFlags::X) != PTEFlags::empty() { "1" } else { "0" },
                if (flags & PTEFlags::U) != PTEFlags::empty() { "1" } else { "0" },
                if (flags & PTEFlags::G) != PTEFlags::empty() { "1" } else { "0" },
                if (flags & PTEFlags::A) != PTEFlags::empty() { "1" } else { "0" },
                if (flags & PTEFlags::D) != PTEFlags::empty() { "1" } else { "0" },
                if valid { pte.ppn().0 } else { 0 }
            );
            
            // Remember the first valid entry to follow
            if valid && !found_valid && level < 3 {
                found_valid = true;
                next_ppn = Some(pte.ppn());
            }
        }
        
        // Follow a valid entry to the next level
        if let Some(next) = next_ppn {
            if level < 3 {
                self.dump_raw_pte_level(next, level + 1);
            }
        }
    }
    
    // Follow a complete PTE chain from root to leaf
    fn follow_pte_chain_from_root(&self) {
        // Find a valid mapping to trace
        for region in self.regions.iter() {
            if let Some(vpn) = region.vpn_range.into_iter().next() {
                println!("\nTRACING COMPLETE PTE CHAIN FOR VPN {:#x}:", vpn.0);
                
                let idxs = vpn.indexes();
                println!("VPN indexes: [{:#x}, {:#x}, {:#x}, {:#x}]", 
                    idxs[0], idxs[1], idxs[2], idxs[3]);
                
                let mut ppn = self.page_table.root_ppn;
                println!("L0: Root PPN = {:#x}", ppn.0);
                
                let mut valid_chain = true;
                
                for (level, &idx) in idxs.iter().enumerate() {
                    let pte = &ppn.get_pte_array()[idx];
                    
                    if !pte.is_valid() {
                        println!("L{}: PTE at index {:#x} is invalid, chain broken", level, idx);
                        valid_chain = false;
                        break;
                    }
                    
                    let next_ppn = pte.ppn();
                    let flags = pte.flags();
                    
                    println!("L{}: PTE[{:#x}] = {:#018x}", level, idx, pte.bits);
                    println!("     PPN = {:#x}, FLAGS = {:#04b}", next_ppn.0, flags.bits());
                    println!("     Physical address = {:#x}", usize::from(PhysAddr::from(next_ppn)));
                    
                    if level < 3 {
                        ppn = next_ppn;
                    }
                }
                
                if valid_chain {
                    println!("Complete mapping traced successfully to physical page");
                }
                
                // Only trace one VPN
                return;
            }
        }
        
        println!("No valid mappings found to trace a PTE chain");
    }

    // 添加一个新的简洁调试方法，仅显示关键映射信息
    pub fn debug_map_info(&self) {
        println!("[MEMORY_MAP_INFO]");
        println!("SATP: {:#x}", self.token());
        println!("Regions count: {}", self.regions.len());
        
        for (i, region) in self.regions.iter().enumerate() {
            let start_va = VirtAddr::from(region.vpn_range.get_start());
            let end_va = VirtAddr::from(region.vpn_range.get_end());
            
            println!("Region #{}: {:#010x} - {:#010x} [{} pages] Type: {:?}", 
                i, 
                start_va.0, 
                end_va.0, 
                region.vpn_range.get_end().0 - region.vpn_range.get_start().0,
                region.map_type);
                
            println!("  Flags: R={} W={} X={} U={} G={}", 
                (region.permission & PTEFlags::R) != PTEFlags::empty(),
                (region.permission & PTEFlags::W) != PTEFlags::empty(),
                (region.permission & PTEFlags::X) != PTEFlags::empty(),
                (region.permission & PTEFlags::U) != PTEFlags::empty(),
                (region.permission & PTEFlags::G) != PTEFlags::empty());
                
            // 打印该区域的第一个和最后一个映射
            if let Some(first_vpn) = region.vpn_range.into_iter().next() {
                if let Some(pte) = self.page_table.find_pte(first_vpn.into()) {
                    let first_ppn = pte.ppn();
                    let first_va: VirtAddr = first_vpn.into();
                    let first_pa: PhysAddr = first_ppn.into();
                    println!("  First page: VPN={:#010x} -> PPN={:#010x}", first_va.0, first_pa.0);
                }
            }
            
            // 获取最后一个VPN
            let mut last_vpn = None;
            for vpn in region.vpn_range {
                last_vpn = Some(vpn);
            }
            
            if let Some(last_vpn) = last_vpn {
                if let Some(pte) = self.page_table.find_pte(last_vpn.into()) {
                    let last_ppn = pte.ppn();
                    let last_va: VirtAddr = last_vpn.into();
                    let last_pa: PhysAddr = last_ppn.into();
                    println!("  Last page: VPN={:#010x} -> PPN={:#010x}", last_va.0, last_pa.0);
                }
            }
        }
        println!("[END_MEMORY_MAP_INFO]");
    }
}

// 映射区域结构
pub struct MapRegion {
    // 虚拟页号范围
    pub vpn_range: VPNRange,
    // 映射类型
    pub map_type: MapType,
    // 映射权限
    pub permission: PTEFlags,
    //framed类型映射的物理页帧
    pub data_frames: BTreeMap<VirtPageNum, FrameTracker>,
}

impl MapRegion {
    // 创建一个新的映射区域
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr,
        map_type: MapType,
        permission: MapPermission,
    ) -> Self {
        // 将权限转换为PTEFlags，并确保设置有效位
        let mut pte_flags = PTEFlags::from_bits(permission.bits()).unwrap();
        pte_flags |= PTEFlags::V; // 添加有效位
        // 计算虚拟页号范围
        let vpn_range = VPNRange::new(start_va.floor(), end_va.ceil());
        // 创建映射区域
        Self {
            vpn_range,
            map_type,
            permission: pte_flags,
            data_frames: BTreeMap::new(),
        }
    }

    pub fn map_one_vpn(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn = match self.map_type {
            MapType::Framed => {
                let frame = frame_alloc().unwrap();
                let ppn = frame.ppn;
                // 将新分配的页帧添加到映射中
                self.data_frames.insert(vpn.into(), frame);
                let va: VirtAddr = vpn.into();
                let pa: PhysAddr = ppn.into();
                println!("[MAP_PAGE] Framed: VPN={:#010x} -> PPN={:#010x}", va.0, pa.0);
                ppn
            }
            MapType::Identity => {
                let ppn = PhysPageNum::from(vpn.0);
                let va: VirtAddr = vpn.into();
                let pa: PhysAddr = ppn.into();
                println!("[MAP_PAGE] Identity: VPN={:#010x} -> PPN={:#010x}", va.0, pa.0);
                ppn
            }
        };
        // 使用已存储的权限标志的副本，确保有效位被设置
        let mut flags = PTEFlags::from_bits(self.permission.bits()).unwrap();
        flags |= PTEFlags::V; // 确保有效位被设置
        println!("[MAP_PAGE] Flags: R={} W={} X={} U={} G={}", 
            (flags & PTEFlags::R) != PTEFlags::empty(),
            (flags & PTEFlags::W) != PTEFlags::empty(),
            (flags & PTEFlags::X) != PTEFlags::empty(),
            (flags & PTEFlags::U) != PTEFlags::empty(),
            (flags & PTEFlags::G) != PTEFlags::empty());
        page_table.map(vpn.into(), ppn, flags);
    }
    pub fn unmap_one_vpn(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        page_table.unmap(vpn.into());
    }
    pub fn map_region(&mut self, page_table: &mut PageTable) {
        for region in self.vpn_range {
            self.map_one_vpn(page_table, region.into());
        }
    }
    pub fn unmap_region(&mut self, page_table: &mut PageTable) {
        for region in self.vpn_range {
            self.unmap_one_vpn(page_table, region.into());
        }
    }
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct MapPermission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MapType {
    // 恒等映射
    Identity,
    // 按需分配
    Framed,
}
