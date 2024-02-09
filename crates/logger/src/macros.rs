#[macro_export]
macro_rules! log {
    ($level:expr, $($attr:expr),+) => {{
        let message = format!("{}", $($attr),+);
        $crate::Logger::log($level, &message);
    }};
}

#[macro_export]
macro_rules! trace {
    ($($attr:expr),+) => {{
        $crate::log!($crate::LogLevel::Trace, $($attr)+);
    }}
}

#[macro_export]
macro_rules! info {
    ($($attr:expr),+) => {{
        $crate::log!($crate::LogLevel::Info,  $($attr)+);
    }}
}

#[macro_export]
macro_rules! warn {
    ($($attr:expr),+) => {{
        $crate::log!($crate::LogLevel::Warn, $($attr)*)
    }};
}

#[macro_export]
macro_rules! error {
    ($($attr:expr),+) => {{
        $crate::log!($crate::LogLevel::Error, $($attr)*);
    }}
}
