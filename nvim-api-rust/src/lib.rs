//!
//! # neovim
//!
//! This crate provides a Rust abstraction over `neovim-sys`, giving native (not msgpack) access to
//! neovim types and functions.
//!
#![deny(unused_extern_crates)]
#![warn(
    box_pointers,
    clippy::all,
    clippy::nursery,
    clippy::pedantic,
    future_incompatible,
    missing_copy_implementations,
    // missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_qualifications
)]

pub mod api;
pub mod key_code;
pub mod option;

pub use neovim_sys as sys;

#[cfg(feature = "lua_test")]
pub mod lua_test;