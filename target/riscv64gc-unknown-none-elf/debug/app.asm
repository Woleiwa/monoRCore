.global apps
    .section .data
    .align 3
apps:
    .quad 0x0
    .quad 0x0
    .quad 7
    .quad app_0_start
    .quad app_1_start
    .quad app_2_start
    .quad app_3_start
    .quad app_4_start
    .quad app_5_start
    .quad app_6_start
    .quad app_6_end

app_0_start:
    .incbin "/home/mesii/code/monoRCore/monoRCore/target/riscv64gc-unknown-none-elf/debug/initproc"
app_0_end:

app_1_start:
    .incbin "/home/mesii/code/monoRCore/monoRCore/target/riscv64gc-unknown-none-elf/debug/sjf1"
app_1_end:

app_2_start:
    .incbin "/home/mesii/code/monoRCore/monoRCore/target/riscv64gc-unknown-none-elf/debug/sjf2"
app_2_end:

app_3_start:
    .incbin "/home/mesii/code/monoRCore/monoRCore/target/riscv64gc-unknown-none-elf/debug/sjf3"
app_3_end:

app_4_start:
    .incbin "/home/mesii/code/monoRCore/monoRCore/target/riscv64gc-unknown-none-elf/debug/sjf4"
app_4_end:

app_5_start:
    .incbin "/home/mesii/code/monoRCore/monoRCore/target/riscv64gc-unknown-none-elf/debug/sjf5"
app_5_end:

app_6_start:
    .incbin "/home/mesii/code/monoRCore/monoRCore/target/riscv64gc-unknown-none-elf/debug/sjftests"
app_6_end:
