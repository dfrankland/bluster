#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod hci;

#[cfg(any(target_os = "windows", target_os = "freebsd"))]
pub mod usb;

#[cfg(target_os = "macos")]
pub mod xpc;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use hci as connection;

#[cfg(any(target_os = "windows", target_os = "freebsd"))]
pub use usb as connection;

#[cfg(target_os = "macos")]
pub use xpc as connection;
