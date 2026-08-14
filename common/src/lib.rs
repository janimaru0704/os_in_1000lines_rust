#![no_std]

pub const SYS_PUTCHAR: i32 = 1;
pub const SYS_GETCHAR: i32 = 2;
pub const SYS_EXIT: i32 = 3;

use core::fmt::{self, Write};

pub fn print(putchar: fn(u8), args: fmt::Arguments) {
    struct Printer {
        putchar: fn(u8),
    }

    impl Write for Printer {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            for byte in s.bytes() {
                (self.putchar)(byte);
            }
            Ok(())
        }
    }

    let mut printer = Printer { putchar };
    printer.write_fmt(args).unwrap();
}

pub const fn align_up(value: usize, align: usize) -> usize {
    value.div_ceil(align) * align
}

pub const fn is_aligned(value: usize, align: usize) -> bool {
    value.is_multiple_of(align)
}

pub unsafe fn memset(dst: *mut u8, value: u8, size: usize) -> *mut u8 {
    unsafe {
        core::ptr::write_bytes(dst, value, size);
    }
    dst
}

pub unsafe fn memcpy(dst: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, size);
    }
    dst
}

pub unsafe fn strcpy(dst: *mut u8, src: *const u8) -> *mut u8 {
    let mut i = 0;

    loop {
        unsafe {
            let c = *src.add(i);
            *dst.add(i) = c;

            if c == 0 {
                break;
            }
        }

        i += 1;
    }
    dst
}

pub unsafe fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut s1 = s1;
    let mut s2 = s2;

    unsafe {
        while *s1 != 0 && *s2 != 0 {
            if *s1 != *s2 {
                break;
            }

            s1 = s1.add(1);
            s2 = s2.add(1);
        }

        *s1 as i32 - *s2 as i32
    }
}
