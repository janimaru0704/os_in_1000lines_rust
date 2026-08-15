use common::print as common_print;

use crate::sbi_call;

pub fn putchar(ch: u8) {
    sbi_call(ch as i32, 0, 0, 0, 0, 0, 0, 1 /* Console PutChar */);
}

pub fn getchar() -> i32 {
    let ret = sbi_call(0, 0, 0, 0, 0, 0, 0, 2);
    ret.error
}

pub fn print_args(args: core::fmt::Arguments) {
    common_print(putchar, args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::console::print_args(
            ::core::format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };

    ($($arg:tt)*) => {
        $crate::print!("{}\n", core::format_args!($($arg)*))
    };
}
