; Phoenix OS 32-bit bootstrap.
;
; GRUB/Multiboot2 hands off to _start in 32-bit protected mode with no paging
; and no long mode enabled -- unlike Limine, which hands off already in 64-bit
; long mode with paging set up. This stub does what Limine used to do for us:
; build minimal page tables (identity-map + higher-half map the first 2GiB
; using 2MiB pages, generous enough to cover early frame allocation regardless
; of how much RAM QEMU is given), enable PAE + long mode + paging, load a
; temporary 64-bit GDT, and far-jump into 64-bit code, which then calls the
; Rust kernel entry point with the Multiboot2 info pointer (passed to us by
; GRUB in EBX) preserved as the first argument.

MULTIBOOT2_MAGIC equ 0xE85250D6
ARCH_I386 equ 0

section .multiboot_header
header_start:
    dd MULTIBOOT2_MAGIC
    dd ARCH_I386
    dd header_end - header_start
    dd -(MULTIBOOT2_MAGIC + ARCH_I386 + (header_end - header_start))

    ; end tag
    dw 0
    dw 0
    dd 8
header_end:

section .bss.boot32
align 4096
p4_table:
    resb 4096
p3_table_low:
    resb 4096
p3_table_high:
    resb 4096
; 2 PD tables covers 2 x 1GiB = 2GiB via 2MiB pages each (P4[511] only has 2
p2_tables:
    resb 4096 * 2
stack_bottom:
    resb 65536
stack_top:

section .rodata.boot32
gdt64:
    dq 0 ; null descriptor
.code: equ $ - gdt64
    dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ; executable, code segment, present, 64-bit
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .text.boot32
bits 32
global _start
extern kernel_main_entry

_start:
    mov esp, stack_top
    ; Preserve Multiboot2 info pointer (GRUB passes it in EBX) across the
    ; whole transition by parking it in a callee-nothing register nobody else
    ; touches below: EDI is untouched by everything until we read it back.
    mov edi, ebx

    call check_multiboot
    call check_cpuid
    call check_long_mode

    call set_up_page_tables
    call enable_paging

    lgdt [gdt64.pointer]
    jmp gdt64.code:long_mode_start

; If eax != multiboot2 magic response value, GRUB didn't load us correctly.
check_multiboot:
    cmp eax, 0x36d76289
    jne .no_multiboot
    ret
.no_multiboot:
    mov al, "M"
    jmp error

; cpuid availability check: try to flip EFLAGS bit 21 (ID bit); if it doesn't
; stick, this CPU has no CPUID instruction at all (impossible on anything
; QEMU emulates, but cheap to check and this is the standard pattern).
check_cpuid:
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 1 << 21
    push eax
    popfd
    pushfd
    pop eax
    push ecx
    popfd
    xor eax, ecx
    jz .no_cpuid
    ret
.no_cpuid:
    mov al, "C"
    jmp error

check_long_mode:
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode

    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29
    jz .no_long_mode
    ret
.no_long_mode:
    mov al, "L"
    jmp error

; Identity-map and higher-half-map the first 2GiB using 2MiB pages.
set_up_page_tables:
    ; P4[0] -> P3 (low, identity mapping)
    mov eax, p3_table_low
    or eax, 0b11 ; present + writable
    mov [p4_table], eax

    ; P4[511] -> P3 (high, higher-half mapping at -2GiB and up)
    mov eax, p3_table_high
    or eax, 0b11
    mov [p4_table + 511 * 8], eax

    ; P3_low[0..2) -> P2 tables 0..2 (identity: 0..2GiB)
    ; P3_high[510..512) -> P2 tables 0..2 (higher half: 0xffffffff80000000 up)
    ; 0xffffffff80000000's P3 index is (0xffffffff80000000 >> 30) & 0x1ff = 510
    mov ecx, 0
.map_p3_entries:
    mov eax, ecx
    imul eax, 4096
    add eax, p2_tables
    or eax, 0b11
    mov ebx, p3_table_low
    mov [ebx + ecx * 8], eax

    mov eax, ecx
    imul eax, 4096
    add eax, p2_tables
    or eax, 0b11
    mov ebx, p3_table_high
    mov [ebx + 510 * 8 + ecx * 8], eax

    inc ecx
    cmp ecx, 2
    jne .map_p3_entries

    ; Fill both P2 tables (2 * 512 = 1024 entries) with 2MiB pages covering
    ; physical addresses 0..2GiB, identical mapping used by both the identity
    ; and higher-half views (they point at the same P2 tables).
    mov ecx, 0
.map_p2_entries:
    mov eax, 0x200000 ; 2MiB
    mul ecx
    or eax, 0b10000011 ; present + writable + huge page
    mov [p2_tables + ecx * 8], eax

    inc ecx
    cmp ecx, 1024
    jne .map_p2_entries

    ret

enable_paging:
    ; Load P4 into CR3.
    mov eax, p4_table
    mov cr3, eax

    ; Enable PAE.
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Set the long mode bit (LME, bit 8) AND no-execute enable (NXE, bit 11)
    ; in EFER (MSR 0xC0000080). Without NXE, PTE bit 63 (NO_EXECUTE) is a
    ; RESERVED bit that must stay 0 -- the Rust kernel's heap init requests
    ; NO_EXECUTE on its pages, and setting a reserved bit is exactly a
    ; "malformed page table" violation, which is what was actually causing
    ; the page fault at heap-init time (not a real allocator/paging bug).
    mov ecx, 0xC0000080
    rdmsr
    or eax, (1 << 8) | (1 << 11)
    wrmsr

    ; Enable paging.
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ret

; Print "ERR: X" where X is the value in al, then halt. Direct VGA text
; buffer write -- doesn't depend on anything we haven't set up yet.
error:
    mov dword [0xb8000], 0x4f524f45
    mov dword [0xb8004], 0x4f3a4f52
    mov dword [0xb8008], 0x4f204f20
    mov byte  [0xb800a], al
    hlt

bits 64
long_mode_start:
    ; Clear the direction flag -- the SysV ABI requires DF=0 on entry to any
    ; compiled function and guarantees it stays that way (compiled code never
    ; emits std/cld around ordinary operations). GRUB/BIOS leaves DF in
    ; whatever state it was already in, not necessarily clear. Left unset,
    ; every `rep movsb`/`movsq`/`stosb` the Rust code's memcpy/memset
    ; compiler-builtins execute runs backwards, corrupting whatever memory
    ; is adjacent to the actual source/destination -- this was the real
    ; cause of the seemingly-random cascading exceptions (different type
    ; every run, nonsensical instruction pointers) seen everywhere a copy
    ; larger than a few bytes was attempted (e.g. Vec::extend_from_slice).
    cld

    ; Clear segment registers -- the GDT has no data segments; zero is a
    ; valid null selector for all of them in long mode.
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; rdi already holds the Multiboot2 info pointer: writing edi in 32-bit
    ; mode zero-extends the upper 32 bits of rdi automatically, so it's
    ; already in place as the first System V AMD64 argument.
    mov rsp, stack_top

    call kernel_main_entry

.hang:
    hlt
    jmp .hang
