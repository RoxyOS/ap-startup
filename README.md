# ap-startup

Start x86 application processors and jump to an entry point.

This crate is intended for `no_std` kernels bringing up SMP.

## Usage

```rust,ignore
use acpi::AcpiTables;
use ap_startup::{Platform, startup_ap};
use x2apic::lapic::LocalApic;

struct KernelPlatform;

impl Platform for KernelPlatform {
    fn sleep_us(us: u64) {
        // TODO: implement a proper delay backend.
        let _ = us;

        const CYCLES: u64 = 10000;

        let mut i = 0;

        while i < CYCLES {
            i += 1;
        }
    }

    fn phys_to_ptr<T>(phys_addr: u64) -> *mut T {
        let _ = phys_addr;
        todo!()
    }
}

extern "C" fn ap_main() -> ! {
    loop {}
}

fn start_aps() {
    let acpi_tables = ; // Your ACPI tables instance.
    let local_apic = ; // The BSP local APIC.
    let l4_table = ; // The top-level page table physical address.
    let ctx = ap_startup::Context {
        acpi_tables,
        current_local_apic: local_apic,
        l4_table,
    };
    start_all_aps::<KernelPlatform, YourACPIHandler>(ap_main, ctx)
        .expect("failed to wake APs");
}
```
