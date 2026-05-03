pub trait Platform {
    const STACK_SIZE: usize;

    fn sleep_us(us: u64);
    fn phys_to_ptr<T>(phys_addr: u64) -> *mut T;
    fn map_memory(virt_addr: u64, phys_addr: u64, size: u64);

    fn write_phys<T>(phys_addr: u64, value: T) {
        unsafe {
            Self::phys_to_ptr::<T>(phys_addr).write_volatile(value);
        }
    }
}
