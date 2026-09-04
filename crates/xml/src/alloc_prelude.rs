//! # Alloc Prelude for `#![no_std]` Compatibility
//!
//! Provides unified imports for `String`, `Vec`, `Box`, `format!`, and `vec!` across
//! standard library (`std`) and bare-metal embedded (`no_std` + `alloc`) targets.

#[cfg(not(feature = "std"))]
pub use alloc::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};

#[cfg(feature = "std")]
pub use std::{
    borrow::ToOwned,
    boxed::Box,
    format,
    string::{String, ToString},
    sync::Arc,
    vec,
    vec::Vec,
};
