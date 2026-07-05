ENTRY(_start)

/* Multiboot2/GRUB loads the ELF's LOAD segments at their physical addresses
   directly -- unlike Limine, which remapped a higher-half kernel for us,
   there's no bootloader-side magic here. KERNEL_VMA_OFFSET is the standard
   -2GiB higher-half convention; every section is linked at a high virtual
   address for `.` but placed at a low physical address via AT(), matching
   where the 32-bit bootstrap's page tables (boot32.asm) actually map it:
   physical 1MiB upward, identity- and higher-half-mapped. */
KERNEL_VMA_OFFSET = 0xffffffff80000000;

PHDRS
{
    /* Without its own segment, .multiboot_header/.boot32 (low addresses, ~1M)
       would otherwise get folded into whichever PT_LOAD segment comes next
       (:text, at a ~-2GiB-higher-half address) since they're never assigned
       a phdr of their own -- ld then computes that segment's file size as
       the span between the two, a bogus multi-exabyte "size". */
    boot    PT_LOAD    FLAGS((1 << 0) | (1 << 1) | (1 << 2)) ; /* R+W+X */
    text    PT_LOAD    FLAGS((1 << 0) | (1 << 2)) ; /* R+X */
    rodata  PT_LOAD    FLAGS((1 << 2)) ;             /* R   */
    data    PT_LOAD    FLAGS((1 << 1) | (1 << 2)) ;  /* R+W */
}

SECTIONS
{
    /* Physical load address: 1MiB, the conventional safe location below the
       kernel that GRUB/Multiboot2 kernels use (below 1MiB is real-mode-era
       reserved memory: BDA, EBDA, video memory, option ROMs). The Multiboot2
       header must be within the first 32KiB of the file for GRUB to find it
       while scanning, so .multiboot_header (emitted by boot32.asm) and the
       32-bit bootstrap .text come first, physically un-relocated (they run
       BEFORE paging/higher-half is active, so they must be usable at their
       load-time physical address, not a high virtual one).*/
    . = 1M;

    .multiboot_header : {
        KEEP(*(.multiboot_header))
    } :boot

    .boot32 : {
        *(.text.boot32)
    } :boot

    /* GDT and page tables the 32-bit bootstrap touches directly via 32-bit
       immediate addresses -- these MUST stay at a low, 32-bit-representable
       address; they can't live in the higher-half .rodata/.bss below (a
       32-bit `mov eax, symbol` can't encode a 0xffffffff80000000+ address). */
    .rodata.boot32 : {
        *(.rodata.boot32)
    } :boot

    .bss.boot32 : {
        *(.bss.boot32)
    } :boot

    /* Everything from here on is higher-half: linked at a high virtual
       address, but AT() pins the physical load address to keep counting up
       from 1MiB, matching what the identity+higher-half page tables built
       in boot32.asm actually cover. */
    . += KERNEL_VMA_OFFSET;

    .text ALIGN(4K) : AT(ADDR(.text) - KERNEL_VMA_OFFSET)
    {
        *(.text .text.*)
    } :text

    . += CONSTANT(MAXPAGESIZE);

    .rodata ALIGN(4K) : AT(ADDR(.rodata) - KERNEL_VMA_OFFSET)
    {
        *(.rodata .rodata.*)
    } :rodata

    . += CONSTANT(MAXPAGESIZE);

    .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_VMA_OFFSET)
    {
        *(.data .data.*)
    } :data

    .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_VMA_OFFSET)
    {
        *(.bss .bss.*)
    } :data

    . = ALIGN(4K);
    /* Physical address just past the kernel's own loaded image -- the frame
       allocator (mem/frame.rs) must never hand out frames below this, since
       Multiboot2's raw memory map (unlike Limine's) doesn't know the kernel
       occupies this range and reports it as ordinary usable RAM. Handing out
       a frame the kernel itself lives in corrupts it live -- this is exactly
       what caused the MALFORMED_TABLE page fault at heap-init time. */
    kernel_physical_end = . - KERNEL_VMA_OFFSET;

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.comment)
    }
}
