use core::arch::global_asm;

use crate::trampoline::{GDT_DESC_ADDR, TRAMPOLINE_DATA_ADDR, TRAMPOLINE_STACK_ADDR};

global_asm!(
    r#"
    .section .text.ap_trampoline, "ax"
    .code16
    .global ap_trampoline_start
ap_trampoline_start:
    cli

    // Reset segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax

    // Load the temporary GDT
    lgdt [{gdt_desc}]

    // Enter protected mode
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ljmp $0x08, $ap_trampoline_32

    .code32
ap_trampoline_32:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    // Load page table
    mov eax, dword ptr [{trampoline_data} + 0]
    mov cr3, eax

    // Long mode stuff
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    // Enable paging
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ljmp $0x18, $ap_trampoline_64

    .code64
ap_trampoline_64:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    // Switch to the AP-specific stack
    mov rsp, qword ptr [{trampoline_stack}]
1:
    mov rax, qword ptr [{trampoline_data} + 8]
    jmp rax

    .global ap_trampoline_end
ap_trampoline_end:
"#,
    gdt_desc = const GDT_DESC_ADDR,
    trampoline_data = const TRAMPOLINE_DATA_ADDR,
    trampoline_stack = const TRAMPOLINE_STACK_ADDR,
);
