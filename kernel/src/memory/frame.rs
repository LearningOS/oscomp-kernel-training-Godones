use crate::config::FRAME_SIZE;
/// 物理页帧管理器
/// 使用位图实现
/// 位图的每一位代表一个物理页帧
use alloc::vec::Vec;
use bitmap_allocator::{BitAlloc, BitAlloc16M};
use spin::{Mutex, Once};

pub static FRAME_ALLOCATOR: Once<Mutex<BitAlloc16M>> = Once::new();

extern "C" {
    fn ekernel();
}

pub fn init_frame_allocator() {
    let start = ekernel as usize;
    let end = crate::config::MEMORY_END;
    info!("memory start:{:#x},end:{:#x}", start, end);
    // 计算页面数量
    let page_start = start / FRAME_SIZE;
    let page_end = end / FRAME_SIZE;
    let page_count = page_end - page_start;
    info!(
        "page start:{:#x},end:{:#x},count:{:#x}",
        page_start, page_end, page_count
    );
    FRAME_ALLOCATOR.call_once(|| Mutex::new(BitAlloc16M::default()));
    FRAME_ALLOCATOR.get().unwrap().lock().insert(0..page_count);
}

#[derive(Debug)]
pub struct FrameTracker {
    id: usize,
}
#[allow(unused)]
fn zero_init_frame(start_addr: usize) {
    unsafe {
        core::ptr::write_bytes(start_addr as *mut u8, 0, FRAME_SIZE);
    }
}
pub fn frame_to_addr(index: usize) -> usize {
    index * FRAME_SIZE + ekernel as usize
}

pub fn addr_to_frame(addr: usize) -> FrameTracker {
    FrameTracker::new((addr - ekernel as usize) / FRAME_SIZE)
}

impl FrameTracker {
    pub fn new(id: usize) -> Self {
        Self { id }
    }
    #[allow(unused)]
    pub fn start(&self) -> usize {
        frame_to_addr(self.id)
    }
    #[allow(unused)]
    pub fn end(&self) -> usize {
        self.start() + FRAME_SIZE
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        trace!("drop frame:{}", self.id);
        let flag = FRAME_ALLOCATOR.get().unwrap().lock().test(self.id);
        if flag {
            panic!("frame {} is not allocated", self.id);
        }
        FRAME_ALLOCATOR.get().unwrap().lock().dealloc(self.id);
    }
}

/// 提供给slab分配器的接口
/// 这些页面需要保持连续
#[no_mangle]
fn alloc_frames(num: usize) -> *mut u8 {
    let start = FRAME_ALLOCATOR
        .get()
        .unwrap()
        .lock()
        .alloc_contiguous(num, 0);
    if start.is_none() {
        return core::ptr::null_mut();
    }
    let start = start.unwrap();
    let start_addr = frame_to_addr(start);
    trace!("slab alloc frame {} start:{:#x}", start, start_addr);
    start_addr as *mut u8
}
/// 提供给slab分配器的接口
#[no_mangle]
fn free_frames(addr: *mut u8, num: usize) {
    let start = (addr as usize - ekernel as usize) / FRAME_SIZE;
    trace!("slab free frame {} start:{:#x}", start, addr as usize);
    FRAME_ALLOCATOR
        .get()
        .unwrap()
        .lock()
        .insert(start..start + num);
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .get()
        .unwrap()
        .lock()
        .alloc()
        .map(FrameTracker::new)
}
#[allow(unused)]
pub fn frames_alloc(count: usize) -> Option<Vec<FrameTracker>> {
    let mut ans = Vec::new();
    for _ in 0..count {
        let id = FRAME_ALLOCATOR.get().unwrap().lock().alloc()?;
        ans.push(FrameTracker::new(id));
    }
    Some(ans)
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        // println!("frame {} start:{:#x}", frame.id, frame.start());
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        // println!("frame {} start:{:#x}", frame.id, frame.start());
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}
