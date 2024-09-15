//! Functions here are from `nvim/api/private/*.h`. Kinda seems like maybe we shouldn't be wrapping
//! these, but they're exported, sooo....
//!
use super::buffer::Buffer;
use crate::{
    api::nvim::{LuaError, NvimString, Object},
    buffer_defs::buf_T,
};
use std::{ffi::c_void, os::raw::c_int};

extern "C" {
    /// Gets the value of a global or local (buffer, window) option.
    ///
    /// * If `opt_type` is `SReq::Win` or `SReq::Buf`, `from` must be a pointer to the window or
    /// buffer.
    /// * `opt_type` should be one `SReq`.
    /// * `name` is option name.
    /// * `error` is an out-pointer to capture any error that might occur during the call.
    ///
    pub fn get_option_from(
        from: *const c_void,
        opt_type: c_int,
        name: NvimString,
        error: *mut LuaError,
    ) -> Object;

    /// Gets a handle to a buffer from the buffer number.
    ///
    /// If the return value is null, an error occurred, so you should check `error` for such.
    ///
    pub fn find_buffer_by_handle(buffer: Buffer, error: *mut LuaError) -> *const buf_T;

    /// https://github.com/neovim/neovim/blob/master/src/nvim/api/vim.c#L682
    pub fn nvim_get_option(name: NvimString, error: *mut LuaError) -> Object;
}
