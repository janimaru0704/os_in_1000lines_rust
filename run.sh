#!/bin/bash

set -e

cargo build

qemu-system-riscv32 \
    -machine virt \
    -bios opensbi-riscv32-generic-fw_dynamic.bin \
    -nographic \
    -serial mon:stdio \
    -no-reboot \
    -d unimp,guest_errors,int,cpu_reset -D qemu.log \
    -kernel target/riscv32i-unknown-none-elf/debug/kernel