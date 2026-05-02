use alloc::boxed::Box;

pub(crate) fn allocate_stack(size: usize) -> u64 {
    let stack_box = Box::leak(alloc::vec![0u8; size].into_boxed_slice());

    stack_box.as_ptr() as u64 + size as u64
}
