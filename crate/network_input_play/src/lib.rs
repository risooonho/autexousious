#![deny(missing_debug_implementations, missing_docs)] // kcov-ignore

//! Provides logic for the network play lobby process.

pub use crate::system::{
    GameInputTickRequestSystem, NetworkInputRequestSystem, NetworkInputRequestSystemDesc,
    NetworkInputResponseSystem, NetworkInputResponseSystemDesc,
};

mod system;
