use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;
use crate::mm::address::{PhysAddr, PhysPageNum, StepByOne, VirtAddr, VirtPageNum};
use crate::mm::frame_allocator::{frame_alloc, FrameTracker};

bitflags! {
    #[derive(PartialEq)]
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}
impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn.0 << 10 | flags.bits() as usize,//将物理页号左移10位，然后或上flags的位,前8位是标志位，后两位是保留位，然后就是ppn0，ppn1，ppn2，ppn3
        }
    }
    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }//创建一个空的页表项
    }
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()//将bits右移10位去掉标志位，然后与((1usize << 44) - 1)进行按位与运算，得到物理页号
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()//将bits转换为PTEFlags
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()//判断页表项是否有效
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()//判断页表项是否可读
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()//判断页表项是否可写
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()//判断页表项是否可执行
    }
}
pub struct PageTable {
    root_ppn: PhysPageNum,//根物理页号
    frames: Vec<FrameTracker>,//用途：管理页表占用的物理页帧，使用FrameTracker包装ppn，这样在PageTable销毁时，会自动释放物理页帧
}
impl PageTable{
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root_ppn: frame.ppn,
            frames: vec![frame],
        }
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)),
            frames: Vec::new(),
        }
    }//从satp中获取根物理页号
    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();//将虚拟页号分解为4个索引值
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;//用于存储查找结果
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];//返回整个页表（512个页表项的数组）
            if i == 3 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {//如果页表项无效，则创建一个新页表项
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlags::V);
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }//在页表中查找页表项，如果存在则返回，否则创建一个新页表项
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];//虽然返回的是整个页表，但是这是数组，一个索引是8字节，所以一个索引是页表项
            if i == 3 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }//叶子节点为空，则返回None
            ppn = pte.ppn();
        }
        result
    }//在页表中查找页表项，如果存在则返回，否则返回None
    #[allow(unused)]
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlags) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }//在页表中映射一个虚拟页号到物理页号，并设置标志位
    #[allow(unused)]
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)//如果找到页表项，则返回页表项，否则返回None,本来是返回&mut PageTableEntry，但是这里返回的是PageTableEntry，所以需要解引用
    }//如果找到页表项，则返回页表项，否则返回None
    pub fn translate_va_pa(&self, va: VirtAddr) -> Option<PhysAddr> {
        self.find_pte(va.clone().floor()).map(|pte| {
            let aligned_pa: PhysAddr = pte.ppn().into();//将页表项的物理页号转换为物理地址,页表项指向的那个物理页的物理地址
            let offset = va.page_offset();//获取虚拟地址的偏移量
            let aligned_pa_usize: usize = aligned_pa.into();//为什么要转换为usize？答：因为物理地址是usize类型，所以需要转换为usize
            (aligned_pa_usize + offset).into()//将物理地址和偏移量相加，然后转换为物理地址
        })
    }//如果找到页表项，则返回物理地址，否则返回None
    
    //组成一个satp，SV48的MODE值为9
    //63      60 59    44 43                  0
    //+---------+--------+---------------------+
    //|  MODE   |  ASID  |        PPN         |
    //+---------+--------+---------------------+
    //     4        16             44
    pub fn token(&self) -> usize {
        9usize << 60 | self.root_ppn.0
    }//返回一个satp，SV48的MODE值为9，PPN为根物理页号
}

///将字节缓冲区从其他地址空间加载到内核空间
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token);//从token中获取根物理页号
    let mut start = ptr as usize;//将指针转换为usize
    let end = start + len;//计算结束地址
    let mut v = Vec::new();//创建一个空的向量
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor();
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step();//虚拟页号加1： 移动到下一页
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end));//比较end和end_va哪个小，取小的那个
        if end_va.page_offset() == 0 {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);//如果end_va的偏移量为0,将物理页的内容添加到vec中
        } else {
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);// 处理到页中间
        }
        //页1：[.....|XXXXX]  （X是要读取的数据）
        //页2：[XXXXX|.....]

        //页1：[....XXXXXXX...]
        start = end_va.into();
    }
    v
}//将虚拟地址转换为物理地址，然后返回一个vec，vec中存储的是物理地址的引用,处理了页表项的跨页问题

///将字符串从其他地址空间加载到内核空间，不使用结束符`\0`
pub fn translated_str(token: usize, ptr: *const u8) -> String {
    let page_table = PageTable::from_token(token);
    let mut string = String::new();
    let mut va = ptr as usize;
    loop {
        let ch: u8 = *(page_table
            .translate_va_pa(VirtAddr::from(va))
            .unwrap()
            .get_mut());
        if ch == 0 {
            break;
        }
        string.push(ch as char);
        va += 1;
    }
    string
}
pub fn translated_ref<T>(token: usize, ptr: *const T) -> &'static T {
    let page_table = PageTable::from_token(token);
    page_table
        .translate_va_pa(VirtAddr::from(ptr as usize))
        .unwrap()
        .get_ref()
}//将虚拟地址转换为物理地址，然后返回一个引用

pub fn translated_refmut<T>(token: usize, ptr: *mut T) -> &'static mut T {
    let page_table = PageTable::from_token(token);
    let va = ptr as usize;
    page_table
        .translate_va_pa(VirtAddr::from(va))
        .unwrap()
        .get_mut()
}//将虚拟地址转换为物理地址，然后返回一个可变引用
pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}//用户缓冲区，存储的是物理地址的引用

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }
    pub fn len(&self) -> usize {
        let mut total: usize = 0;
        for b in self.buffers.iter() {
            total += b.len();
        }
        total
    }
}
impl IntoIterator for UserBuffer {
    type Item = *mut u8;
    type IntoIter = UserBufferIterator;
    fn into_iter(self) -> Self::IntoIter {
        UserBufferIterator {
            buffers: self.buffers,
            current_buffer: 0,
            current_idx: 0,
        }
    }
}

pub struct UserBufferIterator {
    buffers: Vec<&'static mut [u8]>,
    current_buffer: usize,
    current_idx: usize,
}

impl Iterator for UserBufferIterator {
    type Item = *mut u8;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_buffer >= self.buffers.len() {
            None
        } else {
            let r = &mut self.buffers[self.current_buffer][self.current_idx] as *mut _;//第self.current_buffer个u8缓冲区self.current_idx bit元素
            if self.current_idx + 1 == self.buffers[self.current_buffer].len() {//如果self.current_idx + 1等于self.buffers[self.current_buffer].len()，则将self.current_idx设置为0，self.current_buffer设置为self.current_buffer + 1
                self.current_idx = 0;
                self.current_buffer += 1;
            } else {
                self.current_idx += 1;
            }
            Some(r)
        }
    }
}