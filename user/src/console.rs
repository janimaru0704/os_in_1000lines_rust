use common::print as common_print;

use crate::syscall;

pub fn putchar(ch: u8) {
    syscall(common::SYS_PUTCHAR, ch as i32, 0, 0);
}

pub fn getchar() -> i32 {
    syscall(common::SYS_GETCHAR, 0, 0, 0)
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