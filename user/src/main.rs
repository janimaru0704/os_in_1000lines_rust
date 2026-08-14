#![no_std]
#![no_main]

use core::{arch::{asm, naked_asm}, panic::PanicInfo};

pub mod console;
pub mod linker;
pub mod shell;

extern "C" fn exit() -> ! {
    syscall(common::SYS_EXIT, 0, 0, 0);
    loop {}
}

pub fn syscall(sysno: i32, arg0: i32, arg1: i32, arg2: i32) -> i32 {
    let mut a0 = arg0;

    unsafe {
        asm!(
            "ecall",

            inlateout("a0") a0,

            in("a1") arg1,
            in("a2") arg2,
            in("a3") sysno,

            options(nostack),
        );
    }

    a0
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
#[unsafe(naked)]
pub unsafe extern "C" fn start() {
    naked_asm!(
        "la sp, {stack_top}",
        "call main",
        "call {exit}",
        stack_top = sym linker::__stack_top,
        exit = sym exit,
    );
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
