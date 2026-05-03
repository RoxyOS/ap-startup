use core::arch::global_asm;

use crate::trampoline::{GDT_DESC_ADDR, TRAMPOLINE_ADDR, TRAMPOLINE_DATA_ADDR, TRAMPOLINE_STACK_ADDR};

global_asm!(
    r#"
    .section .text.ap_trampoline, "ax"
    .set TRAMPOLINE_BASE, {trampoline_base}
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

    .set ap_trampoline_32_addr, TRAMPOLINE_BASE + (ap_trampoline_32 - ap_trampoline_start)
    push 0x08
    push ap_trampoline_32_addr
    retf

    .code32
ap_trampoline_32:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    // Load values from the shared trampoline workspace before enabling paging,
    // so the 64-bit stage does not need to access low memory anymore.
    mov esp, dword ptr [{trampoline_stack} + 0]
    mov ebx, dword ptr [{trampoline_stack} + 4]
    mov esi, dword ptr [{trampoline_data} + 8]
    mov edi, dword ptr [{trampoline_data} + 12]

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

    .set ap_trampoline_64_addr, TRAMPOLINE_BASE + (ap_trampoline_64 - ap_trampoline_start)
    push 0x18
    push ap_trampoline_64_addr
    retf

    .code64
ap_trampoline_64:
    mov ax, 0x10
    mov ds, ax
    mov es, ax
    mov ss, ax

    // Rebuild the 64-bit stack pointer and entry point from the values cached
    // in registers during the 32-bit stage.
    shl rbx, 32
    mov ebx, ebx
    mov rsp, rbx
    mov esp, esp

    shl rdi, 32
    mov edi, edi
    mov rax, rdi
    mov eax, esi
    jmp rax

    .global ap_trampoline_end
ap_trampoline_end:
    "#,
    gdt_desc = const GDT_DESC_ADDR,
    trampoline_base = const TRAMPOLINE_ADDR,
    trampoline_data = const TRAMPOLINE_DATA_ADDR,
    trampoline_stack = const TRAMPOLINE_STACK_ADDR,
);
