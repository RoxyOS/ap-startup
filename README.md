# ap-startup

Start x86 application processors and jump to an entry point.

This crate is intended for `no_std` kernels bringing up SMP.

## Getting Started

### Implement `Platform`

The crate needs two platform-specific operations:

- a microsecond delay function
- a way to turn a physical address into a writable pointer

```rust,ignore
use ap_startup::Platform;

struct KernelPlatform;

impl Platform for KernelPlatform {
    const STACK_SIZE: usize = 0x400000;

    fn sleep_us(us: u64) {
        let _ = us;
        // TODO: implement a proper delay backend.
        todo!()
    }

    fn phys_to_ptr<T>(phys_addr: u64) -> *mut T {
        let _ = phys_addr;
        // TODO: convert a physical address into a writable virtual pointer.
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
- the top-level page table physical address

```rust,ignore
use acpi::AcpiTables;
use x2apic::lapic::LocalApic;

fn make_context<'a, H: acpi::Handler>(
    acpi_tables: &'a AcpiTables<H>,
    local_apic: &'a mut LocalApic,
    l4_table: u64,
) -> ap_startup::Context<'a, H> {
    ap_startup::Context {
        acpi_tables,
        current_local_apic: local_apic,
        l4_table,
    }
}
```

### 4. Start all APs

```rust,ignore
use ap_startup::start_all_aps;

fn start_aps<H: acpi::Handler>(
    acpi_tables: &AcpiTables<H>,
    local_apic: &mut LocalApic,
    l4_table: u64,
) {
    let ctx = ap_startup::Context {
        acpi_tables,
        current_local_apic: local_apic,
        l4_table,
    };

    start_all_aps::<KernelPlatform, H>(ap_main, ctx)
        .expect("failed to wake APs");
}
```

## Limitations

### Shared trampoline workspace

The crate uses one fixed low-memory trampoline workspace:

- `0x8000` for the trampoline code
- `0x8800` for the temporary GDT
- `0x8840` for the GDT descriptor
- `0x8880` for trampoline startup data
- `0x8900` for the current AP stack top

Because this workspace is shared, AP startup is assumed to be serialized.
This crate is not designed for parallel AP bring-up.

### L4 table address limit

The current trampoline only loads the low 32 bits of `CR3` during the 32-bit
startup stage.

That means the top-level page table physical address must be below `4 GiB`.

### Per-AP stack allocation

Right before each AP is started, the crate allocates and leaks a fresh stack of
size `Platform::STACK_SIZE` and publishes its top into the shared trampoline
workspace.

This is simple and works for early bring-up, but it means:

- each AP gets a leaked startup stack
- stack ownership is currently managed by the crate
- the startup stack is distinct from any later per-CPU runtime stack you may want

## What This Crate Does Not Do

- build or install an AP trampoline binary outside its fixed workspace
- switch APs into their final runtime stack
- wait for AP online handshakes
- initialize per-CPU state
- initialize LAPIC, scheduler, or any higher-level AP runtime state

The caller is still responsible for all of the above.
