#!/bin/bash

set -e

TARGET=riscv32i-unknown-none-elf
BUILD=target/$TARGET/debug

echo "== Build User =="
cargo build -p user

echo "== Create user.bin =="
llvm-objcopy \
    --set-section-flags .bss=alloc,contents \
    -O binary \
    "$BUILD/user" \
    "$BUILD/user.bin"

echo "== Create user.bin.o =="
llvm-objcopy \
    -Ibinary \
    -Oelf32-littleriscv \
    "$BUILD/user.bin" \
    "$BUILD/user.bin.o"

echo "== Build kernel =="
cargo build -p kernel

echo "== Run QEMU =="
qemu-system-riscv32 \
    -machine virt \
    -bios opensbi-riscv32-generic-fw_dynamic.bin \
    -nographic \
    -serial mon:stdio \
    -no-reboot \
    -d unimp,guest_errors,int,cpu_reset -D qemu.log \
    -kernel "$BUILD/kernel"