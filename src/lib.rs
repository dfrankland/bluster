// See https://github.com/SSheldon/rust-objc/pull/75 for updates on issues to do with compiler
// warnings caused by `ATOMIC_USIZE_INIT` being deprecated
#![allow(deprecated)]

mod error;
pub mod gatt;
mod peripheral;
mod uuid;

pub use self::{error::*, peripheral::Peripheral, uuid::*};
