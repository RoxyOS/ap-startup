#![no_std]
use acpi::{AcpiTables, Handler};
use x2apic::lapic::LocalApic;

use crate::{
    error::Result,
    platform::Platform,
    trampoline::{TRAMPOLINE_ADDR, setup_trampoline, update_trampoline_stack},
    wakeup::wakeup_all_aps_with,
};

extern crate alloc;

pub mod error;
pub mod misc;
pub mod platform;
pub mod trampoline;
pub mod trampoline_asm;
pub mod wakeup;

pub struct Context<'a, H: Handler> {
    pub acpi_tables: &'a AcpiTables<H>,
    pub current_local_apic: &'a mut LocalApic,
    pub l4_table: u64,
}

pub type EntryPoint = extern "C" fn() -> !;

pub fn start_all_aps<P: Platform, H: Handler>(
    entry_point: EntryPoint,
    ctx: Context<'_, H>,
) -> Result {
    setup_trampoline::<P, H>(entry_point, &ctx)?;
    wakeup_all_aps_with::<P, H, _>(TRAMPOLINE_ADDR, ctx, || update_trampoline_stack::<P>())
}
