#![no_std]
#![no_main]

pub mod alloc;
pub mod console;
pub mod linker;
pub mod panic;
pub mod process;
pub mod trap;

use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

use crate::process::create_process;

#[repr(C)]
pub struct SbiRet {
    error: isize,
    value: isize,
}

pub fn sbi_call(
    arg0: isize,
    arg1: isize,
    arg2: isize,
    arg3: isize,
    arg4: isize,
    arg5: isize,
    fid: isize,
    eid: isize,
) -> SbiRet {
    let mut a0 = arg0;
    let mut a1 = arg1;

    unsafe {
        asm!(
            "ecall",

            inlateout("a0") a0,
            inlateout("a1") a1,

            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a5") arg5,
            in("a6") fid,
            in("a7") eid,

            options(nostack),
        );
    }

    SbiRet {
        error: a0,
        value: a1,
    }
}

fn kernel_main() -> ! {
    unsafe {
        let bss = &raw mut linker::__bss;
        let bss_end = &raw mut linker::__bss_end;
        common::memset(bss, 0, bss_end as usize - bss as usize);
    }

    write_csr!(stvec, trap::kernel_entry as *const () as usize);

    let idle = create_process(0);
    idle.pid = 0;

    unsafe {
        process::IDLE_PROC = idle;
        process::CURRENT_PROC = idle;
    }

    process::create_process(process::proc_a_entry as *const () as usize);
    process::create_process(process::proc_b_entry as *const () as usize);

    process::yield_cpu();

    panic!("switched to idle process");
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
#[unsafe(naked)]
pub unsafe extern "C" fn boot() {
    naked_asm!(
        "la t0, {stack_top}",
        "mv sp, t0",
        "j {kernel_main}",
        stack_top = sym linker::__stack_top,
        kernel_main = sym kernel_main,
    );
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
