use alloc::vec::Vec;
use bitflags::bitflags;
use crate::mm::address::{PhysAddr, PhysPageNum, VPNRange, VirtAddr, VirtPageNum};
use crate::mm::frame_allocator::{frame_alloc, FrameTracker};
use crate::mm::page_table::{PageTable, PageTableEntry, PTEFlags};
use alloc::collections::BTreeMap;

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
        region.map_region(&mut self.page_table);
        self.regions.push(region);
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
                ppn
            }
            MapType::Identity => {
                let ppn = PhysPageNum::from(vpn.0);
                ppn
            }
        };
        // 使用已存储的权限标志的副本，确保有效位被设置
        let mut flags = PTEFlags::from_bits(self.permission.bits()).unwrap();
        flags |= PTEFlags::V; // 确保有效位被设置
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

#[derive(Clone, Copy, PartialEq)]
pub enum MapType {
    // 恒等映射
    Identity,
    // 按需分配
    Framed,
}
