# ap-startup

Start x86 application processors and jump to an entry point.

This crate is intended for `no_std` kernels bringing up SMP.

## Getting Started

### Implement `Platform`

The crate needs:

- a microsecond delay function
- a way to turn a physical address into a writable pointer
- a way to map memory

```rust,ignore
use ap_startup::platform::Platform;

struct MyPlatform;

impl Platform for MyPlatform {
    const STACK_SIZE: usize = 0x400000;

    fn sleep_us(us: u64) {
        let _ = us;
        // TODO: sleep for `us`
        todo!()
    }

    fn phys_to_ptr<T>(phys_addr: u64) -> *mut T {
        let _ = phys_addr;
        // TODO: convert a physical address into a writable virtual pointer.
        todo!()
    }

    fn map_memory(virt_addr: u64, phys_addr: u64, size: u64) {
        let _ = (virt_addr, phys_addr, size);
        // TODO: map `virt_addr..virt_addr + size` to `phys_addr..phys_addr + size`.
        todo!()
    }
}
```

### Provide an AP entry point

This is the entry function that each AP jumps to.

```rust,ignore
extern "C" fn ap_main() -> ! {
    // Do something
    todo!();
}
```

### Build a `Context`

You need:

- parsed ACPI tables
- the BSP local APIC

```rust,ignore
use ap_startup::Context;

let acpi_tables = todo!(); // your parsed ACPI tables
let local_apic = todo!(); // the BSP local APIC

let ctx = Context {
    acpi_tables,
    current_local_apic: local_apic,
};
```

### Start all APs

```rust,ignore
use ap_startup::start_all_aps;

start_all_aps::<MyPlatform, MyACPIHandler>(ap_main, ctx)
    .expect("failed to wake APs");
```

## Example Usage

If your still confused on how to use it, check out [roxy](https://github.com/RoxyOS/roxy/blob/master/kernel/src/smp/startup.rs)
