#![no_std]
//! Wake up x86 application processors.
//!
//! This crate is intended for `no_std` kernels bringing up SMP.
//!
//! `ap-startup` does not provide the AP trampoline itself. It only wakes all
//! APs and jumps to the provided entry point.
//!
//! `ap-startup` does not:
//! - build or install an AP trampoline
//! - switch APs into protected mode or long mode
//! - allocate AP stacks
//! - wait for AP online handshakes
//! - initialize per-CPU state
//!
//! The caller is responsible for all of the above.
//!
//! # Example
//! ```ignore
//! use acpi::AcpiTables;
//! use ap_startup::{Delay, wakeup_all_aps};
//! use x2apic::lapic::LocalApic;
//!
//! struct PlatformDelay;
//!
//! impl Delay for PlatformDelay {
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
//! }
//!
//! fn wake_aps() {
//!     let acpi_tables = ; // Your ACPI tables instance.
//!     let local_apic = ; // The BSP local APIC.
//!     let entry_point = ; // The AP startup entry point.
//!     wakeup_all_aps::<YourACPIHandler, PlatformDelay>(acpi_tables, local_apic, entry_point)
//!         .expect("failed to wake APs");
//! }
//! ```

use acpi::{
    AcpiTables,
    sdt::madt::{Madt, MadtEntry},
};
use x2apic::lapic::LocalApic;

pub trait Delay {
    fn sleep_us(us: u64);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    NoMadt,
    AddrTooHigh,
    AddrNotAligned,
}

pub type Result<T = ()> = core::result::Result<T, Error>;

pub fn wakeup_all_aps<ACPIHandler: acpi::Handler, APDelay: Delay>(
    acpi_tables: &AcpiTables<ACPIHandler>,
    current_local_apic: &mut LocalApic,
    entry_point: u64,
) -> Result {
    check_entry_point(entry_point)?;

    let madt = acpi_tables.find_table::<Madt>().ok_or(Error::NoMadt)?;
    let current_apic_id = unsafe { current_local_apic.id() };

    for madt_entry in madt.get().entries() {
        match madt_entry {
            MadtEntry::LocalApic(local_apic) => {
                if is_cpu_enabled(local_apic.flags) && local_apic.apic_id as u32 != current_apic_id
                {
                    send_sequence::<APDelay>(
                        current_local_apic,
                        local_apic.apic_id as u32,
                        entry_point,
                    );
                }
            }
            MadtEntry::LocalX2Apic(local_x2apic) => {
                if is_cpu_enabled(local_x2apic.flags) && local_x2apic.x2apic_id != current_apic_id {
                    send_sequence::<APDelay>(
                        current_local_apic,
                        local_x2apic.x2apic_id,
                        entry_point,
                    );
                }
            }
            _ => (),
        }
    }

    Ok(())
}

fn check_entry_point(entry_point: u64) -> Result {
    if entry_point >= 0x100000 {
        Err(Error::AddrTooHigh)
    } else if entry_point & 0xfff != 0 {
        Err(Error::AddrNotAligned)
    } else {
        Ok(())
    }
}

fn is_cpu_enabled(flags: u32) -> bool {
    flags & 1 != 0
}

fn addr_to_sipi_vector(addr: u64) -> u8 {
    (addr >> 12) as u8
}

fn send_sequence<APDelay: Delay>(
    current_local_apic: &mut LocalApic,
    apic_id: u32,
    entry_point: u64,
) {
    let vector = addr_to_sipi_vector(entry_point);

    unsafe {
        current_local_apic.send_init_ipi(apic_id);
        APDelay::sleep_us(10_000);
        current_local_apic.send_sipi(vector, apic_id);
        APDelay::sleep_us(200);
        current_local_apic.send_sipi(vector, apic_id);
        APDelay::sleep_us(200);
    }
}
