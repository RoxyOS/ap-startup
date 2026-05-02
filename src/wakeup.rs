use acpi::{
    AcpiTables, Handler,
    sdt::madt::{Madt, MadtEntry},
};
use x2apic::lapic::LocalApic;

use crate::{
    Context,
    error::{Error, Result},
    platform::Platform,
};

pub fn wakeup_all_aps_with<P: Platform, H: Handler, F>(
    trampoline_addr: u64,
    ctx: Context<'_, H>,
    mut func: F,
) -> Result
where
    F: FnMut(),
{
    check_trampoline(trampoline_addr)?;

    let acpi_tables = ctx.acpi_tables;
    let current_local_apic = ctx.current_local_apic;

    let madt = acpi_tables.find_table::<Madt>().ok_or(Error::NoMadt)?;
    let current_apic_id = unsafe { current_local_apic.id() };

    for madt_entry in madt.get().entries() {
        match madt_entry {
            MadtEntry::LocalApic(local_apic) => {
                if is_cpu_enabled(local_apic.flags) && local_apic.apic_id as u32 != current_apic_id
                {
                    func();
                    send_sequence::<P>(
                        current_local_apic,
                        local_apic.apic_id as u32,
                        trampoline_addr,
                    );
                }
            }
            MadtEntry::LocalX2Apic(local_x2apic) => {
                if is_cpu_enabled(local_x2apic.flags) && local_x2apic.x2apic_id != current_apic_id {
                    func();
                    send_sequence::<P>(current_local_apic, local_x2apic.x2apic_id, trampoline_addr);
                }
            }
            _ => (),
        }
    }

    Ok(())
}

fn check_trampoline(trampoline: u64) -> Result {
    if trampoline >= 0x100000 {
        Err(Error::TrampolineTooHigh)
    } else if trampoline & 0xfff != 0 {
        Err(Error::TrampolineNotAligned)
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

fn send_sequence<P: Platform>(current_local_apic: &mut LocalApic, apic_id: u32, trampoline: u64) {
    let vector = addr_to_sipi_vector(trampoline);

    unsafe {
        current_local_apic.send_init_ipi(apic_id);
        P::sleep_us(10_000);
        current_local_apic.send_sipi(vector, apic_id);
        P::sleep_us(200);
        current_local_apic.send_sipi(vector, apic_id);
        P::sleep_us(200);
    }
}
