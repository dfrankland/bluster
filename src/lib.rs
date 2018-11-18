#![feature(uniform_paths)]

pub mod gatt;
mod peripheral;
mod error;
mod uuid;

pub use peripheral::Peripheral;
pub use error::*;
pub use self::uuid::*;
