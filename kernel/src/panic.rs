#[macro_export]
macro_rules! panic {
    ($($arg:tt)*) => {{
        $crate::println!(
            "PANIC: {}:{}: {}",
            file!(),
            line!(),
            format_args!($($arg)*)
        );

        loop {}
    }};
}
