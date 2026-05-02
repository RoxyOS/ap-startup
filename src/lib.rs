#![no_std]
//! Start x86 application processors and jump to an entry point.
//!
//! This crate is intended for `no_std` kernels bringing up SMP.
//!
//! # Example
//! ```ignore
//! use acpi::AcpiTables;
//! use ap_startup::{Platform, startup_ap};
//! use x2apic::lapic::LocalApic;
//!
//! struct KernelPlatform;
//!
//! impl Platform for KernelPlatform {
//!     fn sleep_us(us: u64) {
//!         // TODO: implement a proper delay backend.
//!         let _ = us;
//!
//!         const CYCLES: u64 = 10000;
//!
//!         let mut i = 0;
//!
//!         while i < CYCLES {
//!             i += 1;
//!         }
//!     }
//!
//!     fn phys_to_ptr<T>(phys_addr: u64) -> *mut T {
//!         let _ = phys_addr;
//!         todo!()
//!     }
//! }
//!
//! extern "C" fn ap_main() -> ! {
//!     loop {}
//! }
//!
//! fn start_aps() {
//!     let acpi_tables = ; // Your ACPI tables instance.
//!     let local_apic = ; // The BSP local APIC.
//!     let l4_table = ; // The top-level page table physical address.
//!     let ctx = ap_startup::Context {
//!         acpi_tables,
//!         current_local_apic: local_apic,
//!         l4_table,
//!     };
//!     startup_ap::<KernelPlatform, YourACPIHandler>(ap_main, ctx)
//!         .expect("failed to wake APs");
//! }
//! ```

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
