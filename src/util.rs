#[macro_export]
macro_rules! vprintln {
    ($args:expr, $($arg:tt)*) => {{
        if $args.verbose() {
            eprintln!($($arg)*);
        }
    }};
}
