use crate::{alloc, console, linker, my_panic, println, user_entry};
use core::arch::{asm, naked_asm};

const PROCS_MAX: usize = 8;
const KERNEL_STACK_SIZE: usize = 8192;

const PROC_UNUSED: i32 = 0;
const PROC_RUNNABLE: i32 = 1;

pub const USER_BASE: usize = 0x1000000;

#[repr(C)]
pub struct Process {
    pub pid: i32,
    pub state: i32,
    pub sp: *mut usize,
    pub page_table: *mut u32,
    pub stack: [u8; KERNEL_STACK_SIZE],
}

impl Process {
    const fn new() -> Self {
        Self {
            pid: 0,
            state: PROC_UNUSED,
            sp: core::ptr::null_mut(),
            page_table: core::ptr::null_mut(),
            stack: [0; KERNEL_STACK_SIZE],
        }
    }
}

static mut PROCS: [Process; PROCS_MAX] = [const { Process::new() }; PROCS_MAX];

pub static mut CURRENT_PROC: *mut Process = core::ptr::null_mut();
pub static mut IDLE_PROC: *mut Process = core::ptr::null_mut();

#[unsafe(naked)]
pub unsafe extern "C" fn switch_context(prev_sp: *mut *mut usize, next_sp: *const *mut usize) {
    naked_asm!(
        "addi sp, sp, -13 * 4",
        "sw ra, 0 * 4(sp)",
        "sw s0, 1 * 4(sp)",
        "sw s1, 2 * 4(sp)",
        "sw s2, 3 * 4(sp)",
        "sw s3, 4 * 4(sp)",
        "sw s4, 5 * 4(sp)",
        "sw s5, 6 * 4(sp)",
        "sw s6, 7 * 4(sp)",
        "sw s7, 8 * 4(sp)",
        "sw s8, 9 * 4(sp)",
        "sw s9, 10 * 4(sp)",
        "sw s10, 11 * 4(sp)",
        "sw s11, 12 * 4(sp)",
        "sw sp, (a0)",
        "lw sp, (a1)",
        "lw ra, 0 * 4(sp)",
        "lw s0, 1 * 4(sp)",
        "lw s1, 2 * 4(sp)",
        "lw s2, 3 * 4(sp)",
        "lw s3, 4 * 4(sp)",
        "lw s4, 5 * 4(sp)",
        "lw s5, 6 * 4(sp)",
        "lw s6, 7 * 4(sp)",
        "lw s7, 8 * 4(sp)",
        "lw s8, 9 * 4(sp)",
        "lw s9, 10 * 4(sp)",
        "lw s10, 11 * 4(sp)",
        "lw s11, 12 * 4(sp)",
        "addi sp, sp, 13 * 4",
        "ret",
    );
}

pub fn create_process(image: *const u8, image_size: usize) -> &'static mut Process {
    for i in 0..PROCS_MAX {
        let proc = unsafe { &mut PROCS[i] };

        if proc.state != PROC_UNUSED {
            continue;
        }
        let stack_top = unsafe { proc.stack.as_mut_ptr().add(KERNEL_STACK_SIZE) };

        let mut sp = stack_top as *mut usize;
        unsafe {
            sp = sp.sub(1);
            sp.write(0); // s11

            sp = sp.sub(1);
            sp.write(0); // s10

            sp = sp.sub(1);
            sp.write(0); // s9

            sp = sp.sub(1);
            sp.write(0); // s8

            sp = sp.sub(1);
            sp.write(0); // s7

            sp = sp.sub(1);
            sp.write(0); // s6

            sp = sp.sub(1);
            sp.write(0); // s5

            sp = sp.sub(1);
            sp.write(0); // s4

            sp = sp.sub(1);
            sp.write(0); // s3

            sp = sp.sub(1);
            sp.write(0); // s2

            sp = sp.sub(1);
            sp.write(0); // s1

            sp = sp.sub(1);
            sp.write(0); // s0

            sp = sp.sub(1);
            sp.write(user_entry as *const () as usize); // ra
        }

        let kernel_base = &raw const linker::__kernel_base as usize;
        let free_ram_end = &raw const linker::__free_ram_end as usize;

        let page_table = alloc::alloc_pages(1) as *mut u32;

        for paddr in (kernel_base..free_ram_end).step_by(alloc::PAGE_SIZE) {
            unsafe {
                alloc::map_page(
                    page_table,
                    paddr,
                    paddr,
                    alloc::PAGE_R | alloc::PAGE_W | alloc::PAGE_X,
                );
            }
        }

        for off in (0..image_size).step_by(alloc::PAGE_SIZE) {
            let page = alloc::alloc_pages(1);

            let remaining = image_size - off;
            let copy_size = alloc::PAGE_SIZE.min(remaining);

            unsafe {
                common::memcpy(page as *mut u8, image.add(off), copy_size);

                alloc::map_page(
                    page_table,
                    USER_BASE + off,
                    page,
                    alloc::PAGE_U | alloc::PAGE_R | alloc::PAGE_W | alloc::PAGE_X,
                );
            }
        }

        proc.pid = i as i32 + 1;
        proc.state = PROC_RUNNABLE;
        proc.sp = sp;
        proc.page_table = page_table;

        return proc;
    }

    my_panic!("no free process slots");
}

fn delay() {
    for _ in 0..3000000 {
        unsafe {
            asm!("nop");
        }
    }
}

pub fn proc_a_entry() -> ! {
    println!("starting process A");
    loop {
        console::putchar(b'A');

        yield_cpu();

        delay();
    }
}

pub fn proc_b_entry() {
    println!("starting process B");
    loop {
        console::putchar(b'B');

        yield_cpu();

        delay();
    }
}

pub fn yield_cpu() {
    unsafe {
        let current = &mut *CURRENT_PROC;
        let mut next = &mut *IDLE_PROC;

        for i in 0..PROCS_MAX {
            let index = (current.pid as usize + i) % PROCS_MAX;
            let proc = &mut PROCS[index];

            if proc.state == PROC_RUNNABLE && proc.pid > 0 {
                next = proc;
                break;
            }
        }

        if core::ptr::eq(current, next) {
            return;
        }

        asm!(
            "sfence.vma",
            "csrw satp, {satp}",
            "sfence.vma",
            "csrw sscratch, {sscratch}",
            satp = in(reg) (alloc::SATP_SV32 | (next.page_table as usize / alloc::PAGE_SIZE)),
            sscratch = in(reg) (next.stack.as_ptr().add(KERNEL_STACK_SIZE) as usize),
        );

        let prev = current;
        CURRENT_PROC = next;
        switch_context(&mut prev.sp, &next.sp);
    }
}
