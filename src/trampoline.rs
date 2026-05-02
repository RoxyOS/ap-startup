use core::ptr::{addr_of, copy_nonoverlapping};

use acpi::Handler;

use crate::{
    Context, EntryPoint,
    error::{Error, Result},
    misc::allocate_stack,
    platform::Platform,
};

pub const TRAMPOLINE_ADDR: u64 = 0x8000;
pub const GDT_ADDR: u64 = 0x8800;
pub const GDT_DESC_ADDR: u64 = 0x8840;
pub const TRAMPOLINE_DATA_ADDR: u64 = 0x8880;
pub const TRAMPOLINE_STACK_ADDR: u64 = 0x8900;

#[repr(C)]
struct TrampolineData {
    pub l4_table: u64,
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

pub fn setup_trampoline<P: Platform, H: Handler>(
    entry_point: EntryPoint,
    ctx: &Context<'_, H>,
) -> Result {
    let entry_point = entry_point as *const () as u64;
    let l4_table = ctx.l4_table;

    let trampoline_data = TrampolineData {
        l4_table,
        entry_point,
    };

    if l4_table >= 0x10_000_000 {
        return Err(Error::L4TableAddrTooHigh);
    }

    copy_trampoline::<P>();
    setup_trampoline_gdt::<P>();
    setup_trampoline_data::<P>(trampoline_data);

    Ok(())
}

// Allocate a fresh per-AP stack and publish its top into the shared trampoline
// workspace right before waking the next AP.
pub fn update_trampoline_stack<P: Platform>() {
    let stack_top = allocate_stack(P::STACK_SIZE);
    P::write_phys(TRAMPOLINE_STACK_ADDR, stack_top);
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
    // null, 32-bit code, 32-bit data, 64-bit code
    let gdt: [u64; 4] = [
        0x0000_0000_0000_0000,
        0x00cf_9a00_0000_ffff,
        0x00cf_9200_0000_ffff,
        0x00af_9a00_0000_ffff,
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
