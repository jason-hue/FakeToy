mod page_table;
mod heap_allocator;
mod address;
mod frame_allocator;
pub use heap_allocator::heap_test;
pub use frame_allocator::{frame_allocator_alloc_more_test,frame_allocator_test};
pub fn mm_init(){
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
}