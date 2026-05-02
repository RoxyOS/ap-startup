use acpi::{
    AcpiTables,
    sdt::madt::{Madt, MadtEntry},
};
use x2apic::lapic::LocalApic;

use crate::{
    error::{Error, Result},
    platform::Platform,
};

pub fn wakeup_all_aps<ACPIHandler: acpi::Handler, APDelay: Platform>(
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

fn send_sequence<P: Platform>(current_local_apic: &mut LocalApic, apic_id: u32, entry_point: u64) {
    let vector = addr_to_sipi_vector(entry_point);

    unsafe {
        current_local_apic.send_init_ipi(apic_id);
        P::sleep_us(10_000);
        current_local_apic.send_sipi(vector, apic_id);
        P::sleep_us(200);
        current_local_apic.send_sipi(vector, apic_id);
        P::sleep_us(200);
    }
}
