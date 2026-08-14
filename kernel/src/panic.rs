#[macro_export]
macro_rules! my_panic {
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
