use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(windows)] {
        pub mod win32;
    } else if #[cfg(unix)] {
        pub mod x11;
    }
}