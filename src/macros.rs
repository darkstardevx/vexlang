#[macro_export]
macro_rules! note {
    ($($arg:tt)*) => {
        eprintln!("\x1b[36m[NOTE]\x1b[0m {}", format!($($arg)*));
    }
}
