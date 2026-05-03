use core::arch::global_asm;

use crate::trampoline::{
    GDT_DESC_ADDR, STARTUP_CONFIRMATION_ADDR, TRAMPOLINE_ADDR, TRAMPOLINE_DATA_ADDR,
    TRAMPOLINE_STACK_ADDR,
};

global_asm!(
    r#"
    .section .text.ap_trampoline, "ax"
    .set TRAMPOLINE_BASE, {trampoline_base}
    .code16
    .global ap_trampoline_start
ap_trampoline_start:
    cli
    cld

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

    .set ap_trampoline_32_addr, TRAMPOLINE_BASE + (ap_trampoline_32 - ap_trampoline_start)
    .byte 0xea
    .word ap_trampoline_32_addr
    .word 0x0018

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
    or eax, (1 << 10) | (1 << 9) | (1 << 5) | (1 << 4)
    mov cr4, eax

    mov ecx, 0xC0000080
    xor edx, edx
    rdmsr
    or eax, (1 << 11) | (1 << 8)
    wrmsr

    // Enable paging
    mov eax, cr0
    and eax, ~(1 << 2)
    or eax, (1 << 31) | (1 << 16) | (1 << 1)
    mov cr0, eax

    .set ap_trampoline_64_addr, TRAMPOLINE_BASE + (ap_trampoline_64 - ap_trampoline_start)
    .byte 0xea
    .long ap_trampoline_64_addr
    .word 0x0008

    .code64
ap_trampoline_64:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    mov rsp, qword ptr [{trampoline_stack}]
    mov qword ptr [{startup_confirmation}], 1
    mov rax, qword ptr [{trampoline_data} + 8]
    call rax
    ud2

    .global ap_trampoline_end
ap_trampoline_end:
    "#,
    gdt_desc = const GDT_DESC_ADDR,
    trampoline_base = const TRAMPOLINE_ADDR,
    trampoline_data = const TRAMPOLINE_DATA_ADDR,
    startup_confirmation = const STARTUP_CONFIRMATION_ADDR,
    trampoline_stack = const TRAMPOLINE_STACK_ADDR,
);
