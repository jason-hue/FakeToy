OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;
SECTIONS
{
	. = BASE_ADDRESS;
	skernel = .;
	stext = .;
    .text : {
        *(.text.boot)
        . = ALIGN(4K);
        *(.text .text.*)
    }
    . = ALIGN(4K);
    etext = .;
	srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    erodata = .;
    start_with_stack = .;
    .data : {
        *(.data.stack)
        sdata = .;
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    idmap_pg_dir = .;
    . += 2*4096;
    idmap_pg_end = .;
    edata = .;
	sbss = .;
    .bss : {
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    ebss = .;
    ekernel = .;
}

