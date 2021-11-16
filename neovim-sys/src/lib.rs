//!
//! # neovim-sys
//!
//! This crate provides type-wrappers and functions to allow for directly (as in "not going through
//! the msgpack API) calling neovim functions.
//!
#![deny(unused_extern_crates)]
#![warn(
    box_pointers,
    clippy::all,
    clippy::nursery,
    clippy::pedantic,
    future_incompatible,
    missing_copy_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]
#![allow(non_camel_case_types)]

pub mod api;
pub mod types;

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn init_logger() {
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .with_timestamps(true)
        .init()
        .ok();
}
