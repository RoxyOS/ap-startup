use core::ptr::{addr_of, copy_nonoverlapping};

use x86_64::registers::control::Cr3;

use crate::{
    EntryPoint,
    error::{Error, Result},
    misc::allocate_stack,
    platform::Platform,
};

pub const TRAMPOLINE_ADDR: u64 = 0x8000;
pub const GDT_ADDR: u64 = 0x8800;
pub const GDT_DESC_ADDR: u64 = 0x8840;
pub const TRAMPOLINE_DATA_ADDR: u64 = 0x8880;
pub const TRAMPOLINE_STACK_ADDR: u64 = 0x8900;
// The AP will write to this address once it has started.
pub const STARTUP_CONFIRMATION_ADDR: u64 = 0x8908;
pub const TRAMPOLINE_PAGE_SIZE: u64 = 0x1000;
const STARTUP_CONFIRMATION_RETRIES: usize = 100_000;

#[repr(C)]
struct TrampolineData {
    pub current_l4_table: u64,
    pub entry_point: u64,
}

#[repr(C, packed)]
struct GdtDescriptor {
    limit: u16,
    base: u32,
}

// Symbols from the global_asm! for locating the trampoline
unsafe extern "C" {
    static ap_trampoline_start: u8;
    static ap_trampoline_end: u8;
}

const _: () = {
    assert!(GDT_ADDR >= TRAMPOLINE_ADDR);
    assert!(GDT_DESC_ADDR >= TRAMPOLINE_ADDR);
    assert!(TRAMPOLINE_DATA_ADDR >= TRAMPOLINE_ADDR);
    assert!(TRAMPOLINE_STACK_ADDR >= TRAMPOLINE_ADDR);
    assert!(STARTUP_CONFIRMATION_ADDR >= TRAMPOLINE_ADDR);

    assert!(GDT_ADDR < TRAMPOLINE_ADDR + TRAMPOLINE_PAGE_SIZE);
    assert!(GDT_DESC_ADDR < TRAMPOLINE_ADDR + TRAMPOLINE_PAGE_SIZE);
    assert!(TRAMPOLINE_DATA_ADDR < TRAMPOLINE_ADDR + TRAMPOLINE_PAGE_SIZE);
    assert!(TRAMPOLINE_STACK_ADDR < TRAMPOLINE_ADDR + TRAMPOLINE_PAGE_SIZE);
    assert!(STARTUP_CONFIRMATION_ADDR < TRAMPOLINE_ADDR + TRAMPOLINE_PAGE_SIZE);
};

pub(crate) fn setup_trampoline<P: Platform>(entry_point: EntryPoint) -> Result {
    let entry_point = entry_point as *const () as u64;

    let (l4_table_frame, _) = Cr3::read();
    let current_l4_table = l4_table_frame.start_address().as_u64();

    let trampoline_data = TrampolineData {
        current_l4_table,
        entry_point,
    };

    if current_l4_table >= 0x1_0000_0000 {
        return Err(Error::L4TableAddrTooHigh);
    }

    P::map_memory(TRAMPOLINE_ADDR, TRAMPOLINE_ADDR, TRAMPOLINE_PAGE_SIZE);
    copy_trampoline::<P>();
    setup_trampoline_gdt::<P>();
    setup_trampoline_data::<P>(trampoline_data);

    Ok(())
}

pub(crate) fn prepare_next_ap<P: Platform>() {
    let stack_top = allocate_stack(P::STACK_SIZE);
    P::write_phys(TRAMPOLINE_STACK_ADDR, stack_top);
    P::write_phys(STARTUP_CONFIRMATION_ADDR, 0_u64);
}

pub(crate) fn wait_for_ap_startup<P: Platform>() -> Result {
    for _ in 0..STARTUP_CONFIRMATION_RETRIES {
        let confirmation =
            unsafe { P::phys_to_ptr::<u64>(STARTUP_CONFIRMATION_ADDR).read_volatile() };
        if confirmation != 0 {
            return Ok(());
        }
        P::sleep_us(1);
    }

    Err(Error::StartupTimeout)
}

fn setup_trampoline_data<P: Platform>(data: TrampolineData) {
    P::write_phys(TRAMPOLINE_DATA_ADDR, data);
}

fn copy_trampoline<P: Platform>() {
    unsafe {
        let start = addr_of!(ap_trampoline_start);
        let end = addr_of!(ap_trampoline_end);
        let len = end.offset_from(start) as usize;

        copy_nonoverlapping(start, P::phys_to_ptr(TRAMPOLINE_ADDR), len);
    }
}

fn setup_trampoline_gdt<P: Platform>() {
    // null, 64-bit code, flat data, 32-bit code
    let gdt: [u64; 4] = [
        0x0000_0000_0000_0000,
        0x00af_9a00_0000_ffff,
        0x00cf_9200_0000_ffff,
        0x00cf_9a00_0000_ffff,
    ];

    unsafe {
        copy_nonoverlapping(gdt.as_ptr(), P::phys_to_ptr(GDT_ADDR), gdt.len());
    }

    let gdt_descriptor = GdtDescriptor {
        base: GDT_ADDR as u32,
        limit: (size_of::<[u64; 4]>() - 1) as u16,
    };

    P::write_phys(GDT_DESC_ADDR, gdt_descriptor);
}

#[cfg(test)]
mod tests {
    use super::{STARTUP_CONFIRMATION_ADDR, TRAMPOLINE_ADDR, TRAMPOLINE_PAGE_SIZE};

    #[test]
    fn trampoline_workspace_fits_in_one_page() {
        let workspace_end = STARTUP_CONFIRMATION_ADDR + size_of::<u64>() as u64;
        assert!(workspace_end <= TRAMPOLINE_ADDR + TRAMPOLINE_PAGE_SIZE);
    }

    #[test]
    fn temporary_gdt_selectors_match_trampoline_jumps() {
        let gdt: [u64; 4] = [
            0x0000_0000_0000_0000,
            0x00af_9a00_0000_ffff,
            0x00cf_9200_0000_ffff,
            0x00cf_9a00_0000_ffff,
        ];

        let long_mode_code = gdt[1];
        let data = gdt[2];
        let protected_mode_code = gdt[3];

        assert_eq!(long_mode_code, 0x00af_9a00_0000_ffff);
        assert_eq!(data, 0x00cf_9200_0000_ffff);
        assert_eq!(protected_mode_code, 0x00cf_9a00_0000_ffff);
    }
}
