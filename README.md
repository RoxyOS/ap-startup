# ap-startup

Wake up x86 application processors.

This crate is intended for `no_std` kernels bringing up SMP.

## Notes

`ap-startup` does not provide the AP trampoline itself.
It only wakes all APs and jumps to the provided entry point.

`ap-startup` does not:
    
- build or install an AP trampoline
- switch APs into protected mode or long mode
- allocate AP stacks
- wait for AP online handshakes
- initialize per-CPU state

The caller is responsible for all of the above.

## Usage

```rust,ignore
use acpi::AcpiTables;
use ap_startup::{Delay, wakeup_all_aps};
use x2apic::lapic::LocalApic;

struct PlatformDelay;

impl Delay for PlatformDelay {
    fn sleep_us(us: u64) {
        // TODO: implement a proper delay backend.
        let _ = us;

        const CYCLES: u64 = 10000;

        let mut i = 0;

        while i < CYCLES {
            i += 1;
        }
    }
}

fn wake_aps() {
    let acpi_tables = ; // Your ACPI tables instance.
    let local_apic = ; // The BSP local APIC.
    let entry_point = ; // The AP startup entry point.
    wakeup_all_aps::<YourACPIHandler, PlatformDelay>(acpi_tables, local_apic, entry_point)
        .expect("failed to wake APs");
}
```
