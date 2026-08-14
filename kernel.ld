ENTRY(boot)

SECTIONS {
    . = 0x80200000;
    __kernel_base = .;

    .text : {
        KEEP(*(.text.boot));
        *(.text .text.*);
    }

    .rodata : ALIGN(4) {
        *(.rodata .rodata.*);
    }

    .data : ALIGN(4) {
        *(.data .data.*);
    }

    .bss : ALIGN(4) {
        __bss = .;
        *(.bss .bss.* .sbss .sbss.*);
        __bss_end = .;
    }

    . = ALIGN(4);
    . += 128 * 1024;    /* 128KB */
    __stack_top = .;

    . = ALIGN(4096);
    __free_ram = .;
    . += 64 * 1024 * 1024; /* 64MB */
    __free_ram_end = .;
}

/* user application */
__user_app_start = _binary_target_riscv32i_unknown_none_elf_debug_user_bin_start;
__user_app_end = _binary_target_riscv32i_unknown_none_elf_debug_user_bin_end;