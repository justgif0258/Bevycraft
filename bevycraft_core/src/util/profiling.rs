#[macro_export]
macro_rules! profile_scope {
    ($name:expr) => {
        #[cfg(feature = "profile")]
        let _scope = tracing::info_span!($name).entered();
    };
    ($name:expr, $($field:tt)*) => {
        #[cfg(feature = "profile")]
        let _scope = tracing::info_span!($name, $($field)*).entered();
    }
}

#[macro_export]
macro_rules! profile_event {
    ($msg:expr) => {
        #[cfg(feature = "profile")]
        tracing::event!(tracing::Level::INFO, message = $msg);
    };
}

#[macro_export]
macro_rules! profile_plot {
    ($name:literal, $value:expr) => {
        #[cfg(feature = "profile")]
        {
            ::tracing::trace!(target: "tracy", plot = $name, value = ($value as f64));
        }
    };
}

#[macro_export]
macro_rules! profile_frame {
    () => {
        #[cfg(feature = "profile")]
        tracy_client::frame_mark();
    };
}