#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        $crate::logger::core::log($crate::logger::model::LogLevel::Info, format!($($arg)+))
    }
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)+) => {
        $crate::logger::core::log($crate::logger::model::LogLevel::Warn, format!($($arg)+))
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        $crate::logger::core::log($crate::logger::model::LogLevel::Error, format!($($arg)+))
    }
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        $crate::logger::core::log($crate::logger::model::LogLevel::Debug, format!($($arg)+))
    }
}
