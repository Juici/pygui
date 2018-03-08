#[macro_export]
macro_rules! assert_pyval {
    ($cond:expr, $($arg:tt)+) => (
        if !$cond {
            return Err(exc::ValueError::new(format!($($arg)+)));
        }
    );
}