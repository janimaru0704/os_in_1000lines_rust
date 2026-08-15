#![no_std]
#![no_main]

pub mod alloc;
pub mod console;
pub mod file_system;
pub mod linker;
pub mod panic;
pub mod process;
pub mod trap;
pub mod virtio;

use core::arch::{asm, naked_asm};
use core::panic::PanicInfo;

const SSTATUS_SPIE: u32 = 1 << 5;

#[repr(C)]
pub struct SbiRet {
    error: i32,
    value: i32,
}

pub fn sbi_call(
    arg0: i32,
    arg1: i32,
    arg2: i32,
    arg3: i32,
    arg4: i32,
    arg5: i32,
    fid: i32,
    eid: i32,
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

    write_csr!(stvec, trap::kernel_entry as *const () as u32);

    virtio::virtio_blk_init();

    file_system::fs_init();

    let idle = process::create_process(core::ptr::null(), 0);
    idle.pid = 0;

    unsafe {
        process::IDLE_PROC = idle;
        process::CURRENT_PROC = idle;
    }

    let user_app_start = &raw const linker::__user_app_start as *const u8;
    let user_app_end = &raw const linker::__user_app_end as *const u8;

    process::create_process(
        user_app_start,
        user_app_end as usize - user_app_start as usize,
    );

    process::yield_cpu();

    my_panic!("switched to idle process");
}

#[unsafe(naked)]
pub unsafe extern "C" fn user_entry() -> ! {
    naked_asm!(
        "li t0, {sepc}",
        "csrw sepc, t0",
        "li t0, {sstatus}",
        "csrw sstatus, t0",
        "sret",
        sepc = const process::USER_BASE,
        sstatus = const SSTATUS_SPIE,
    );
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.boot")]
#[unsafe(naked)]
pub unsafe extern "C" fn boot() {
    naked_asm!(
        "la sp, {stack_top}",
        "j {kernel_main}",
        stack_top = sym linker::__stack_top,
        kernel_main = sym kernel_main,
    );
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
