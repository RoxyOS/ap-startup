use alloc::alloc::{Layout, alloc_zeroed};

pub(crate) fn allocate_stack(size: usize) -> u64 {
    let layout = Layout::from_size_align(size, 16).expect("invalid AP stack layout");
    let stack_ptr = unsafe { alloc_zeroed(layout) };

    assert!(!stack_ptr.is_null(), "failed to allocate AP stack");

    stack_ptr as u64 + size as u64
}
