#![feature(uniform_paths)]

pub mod gatt;
mod peripheral;
mod error;

pub use peripheral::Peripheral;
pub use error::*;
