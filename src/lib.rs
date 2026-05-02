//! Start x86 application processors and jump to an entry point.
//!
//! This crate is intended for `no_std` kernels bringing up SMP.
//!
//! ## Getting Started
//!
//! ### Implement `Platform`
//!
//! The crate needs two platform-specific operations:
//!
//! - a microsecond delay function
//! - a way to turn a physical address into a writable pointer
//!
//! ```rust,ignore
//! struct MyPlatform;
//!
//! impl Platform for MyPlatform {
//!     const STACK_SIZE: usize = 0x400000;
//!
//!     fn sleep_us(us: u64) {
//!         let _ = us;
//!         // TODO: implement a proper delay backend.
//!         todo!()
//!     }
//!
//!     fn phys_to_ptr<T>(phys_addr: u64) -> *mut T {
//!         let _ = phys_addr;
//!         // TODO: convert a physical address into a writable virtual pointer.
//!         todo!()
//!     }
//! }
//! ```
//!
//! ### Provide an AP entry point
//!
//! This is the entry function that each AP jumps to.
//!
//! ```rust,ignore
//! extern "C" fn ap_main() -> ! {
//!     // Do something
//!     todo!();
//! }
//! ```
//!
//! ### Build a `Context`
//!
//! You need:
//!
//! - parsed ACPI tables
//! - the BSP local APIC
//! - the top-level page table physical address
//!
//! ```rust,ignore
//! let acpi_tables = todo!(); // your parsed ACPI tables
//! let local_apic = todo!(); // the BSP local APIC
//! let current_l4_table = todo!(); // the physical address of the current L4 page table
//!                                // this page table must map the trampoline workspace
//!                                // and the `ap_main` entry point after paging is enabled
//!
//! let ctx = Context {
//!     acpi_tables,
//!     current_local_apic: local_apic,
//!     current_l4_table,
//! };
//! ```
//!
//! ### Start all APs
//!
//! ```rust,ignore
//! start_all_aps::<MyPlatform, MyACPIHandler>(ap_main, ctx)
//!     .expect("failed to wake APs");
//! ```
//!
//! ## Limitations
//!
//! ### Shared trampoline workspace
//!
//! The crate uses one fixed low-memory trampoline workspace:
//!
//! - `0x8000` for the trampoline code
//! - `0x8800` for the temporary GDT
//! - `0x8840` for the GDT descriptor
//! - `0x8880` for trampoline startup data
//! - `0x8900` for the current AP stack top
//!
//! Because this workspace is shared, AP startup is assumed to be serialized.
//! This crate is not designed for parallel AP bring-up.
//!
//! ### L4 table address limit
//!
//! The current trampoline only loads the low 32 bits of `CR3` during the 32-bit
//! startup stage.
//!
//! That means the top-level page table physical address must be below `4 GiB`.
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
    pub current_l4_table: u64,
}

pub type EntryPoint = extern "C" fn() -> !;

pub fn start_all_aps<P: Platform, H: Handler>(
    entry_point: EntryPoint,
    ctx: Context<'_, H>,
) -> Result {
    setup_trampoline::<P, H>(entry_point, &ctx)?;
    wakeup_all_aps_with::<P, H, _>(TRAMPOLINE_ADDR, ctx, || update_trampoline_stack::<P>())
}
