#![no_std]
#![no_main]

use core::{arch::naked_asm, panic::PanicInfo};

pub mod linker;
pub mod shell;

fn exit() -> ! {
    loop {}
}

#[allow(dead_code)]
fn putchar(_ch: u8) {
    /* 後で実装する */
    unimplemented!("後で実装する");
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
