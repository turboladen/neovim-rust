//!
//! This module contains function wrappers for neovim functions defined in
//! `neovim/src/nvim/api/vim.c`.
//!
use super::{Buffer, Error};
use neovim_sys::{
    api::{
        nvim::{self, Dictionary, LuaError, NvimString, Object, ObjectType},
        private,
    },
    option::{self, OptionFlags, SOpt, SReq},
};
use std::{
    ffi::{c_void, CStr},
    mem::MaybeUninit,
    os::raw::c_char,
};

/// Simliar to `nvim_get_option` and `:set {option}?`.
///
/// # Errors
///
/// This errors if:
///
/// * If `name` contains a nul byte
/// * If vim returns an error
///
pub fn nvim_get_option(name: &str) -> Result<Object, Error> {
    let api_name = NvimString::new(name)?;
    let mut out_err = LuaError::default();

    let object = unsafe { private::nvim_get_option(api_name, &mut out_err) };

    if out_err.is_err() {
        Err(Error::from(out_err))
    } else {
        Ok(object)
    }
}

/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
pub fn nvim_get_global_option(name: &str) -> Result<Object, Error> {
    _get_option_strict(name, SReq::Global, std::ptr::null())
}

fn _get_option_strict(
    name: &str,
    option_type: SReq,
    option_source: *const c_void,
) -> Result<Object, Error> {
    let api_name = NvimString::new(name)?;
    let mut numval = MaybeUninit::<i64>::uninit();
    let mut stringval = MaybeUninit::<*const c_char>::uninit();

    let flags = unsafe {
        option::get_option_value_strict(
            api_name.as_ptr(),
            numval.as_mut_ptr(),
            stringval.as_mut_ptr(),
            option_type as i32,
            option_source,
        )
    };

    if flags == 0 {
        return Err(Error::Raw(format!("Unknown type for option '{}'", name)));
    }

    // If we have a number...
    if !numval.as_ptr().is_null() {
        let numval_init = unsafe { numval.assume_init() };

        if (flags & SOpt::Bool as i32) != 0 {
            return Ok(Object::from(numval_init == 1));
        } else if (flags & SOpt::Num as i32) != 0 {
            return Ok(Object::from(numval_init));
        }
    }

    if !stringval.as_ptr().is_null() {
        let stringval_init = unsafe { stringval.assume_init() };

        if (flags & SOpt::String as i32) != 0 {
            let cstring = unsafe { CStr::from_ptr(stringval_init) };

            return Ok(Object::from(NvimString::new(cstring.to_bytes())?));
        }
    }

    Err(Error::Raw(format!("Unknown type for option '{}'", name)))
}

/// Just like `:setglobal`.
///
/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
pub fn nvim_set_global_option<T>(name: &str, value: T) -> Result<(), Error>
where
    Object: From<T>,
{
    _set_option(name, OptionFlags::OptGlobal as i32, Object::from(value))
}

/// Just like `:set`.
///
/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
pub fn nvim_set_option<T>(name: &str, value: T) -> Result<(), Error>
where
    Object: From<T>,
{
    _set_option(
        name,
        OptionFlags::OptGlobal as i32 & OptionFlags::OptLocal as i32, // Should be 0
        Object::from(value),
    )
}

// Allowing this clippy for now just because passing in a value seems a nicer API. If I start
// ending up cloning values when calling, fix it.
#[allow(clippy::needless_pass_by_value)]
fn _set_option(name: &str, scope: i32, value: Object) -> Result<(), Error> {
    let name_ptr = NvimString::new(name).unwrap();

    let maybe_err = match value.object_type() {
        ObjectType::kObjectTypeBoolean => unsafe {
            option::set_option_value(
                name_ptr.as_ptr(),
                if value.as_boolean_unchecked() { 1 } else { 0 },
                std::ptr::null(),
                scope,
            )
        },
        ObjectType::kObjectTypeInteger => unsafe {
            option::set_option_value(
                name_ptr.as_ptr(),
                value.as_integer_unchecked(),
                std::ptr::null(),
                scope,
            )
        },
        ObjectType::kObjectTypeString => {
            let s = value.as_string_unchecked();

            unsafe {
                option::set_string_option_direct(
                    name_ptr.as_ptr(),
                    -1, // <- -1 means use the name to look up the value
                    s.to_string_lossy().as_ref().as_ptr(),
                    scope,
                    neovim_sys::globals::Sid::ApiClient as i32,
                );
                std::ptr::null()
            }
        }
        t => {
            eprintln!("Got a {:?}", t);
            std::ptr::null()
        }
    };

    if maybe_err.is_null() {
        Ok(())
    } else {
        let e = unsafe { CStr::from_ptr(maybe_err) };
        Err(Error::Raw(e.to_string_lossy().into()))
    }
}

/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
pub fn nvim_get_var(name: &str) -> Result<Object, Error> {
    _get_var(name, |api_name, out_err| unsafe {
        nvim::nvim_get_var(api_name, out_err)
    })
}

/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
pub fn nvim_get_vvar(name: &str) -> Result<Object, Error> {
    _get_var(name, |api_name, out_err| unsafe {
        nvim::nvim_get_vvar(api_name, out_err)
    })
}

/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
fn _get_var<F>(name: &str, getter: F) -> Result<Object, Error>
where
    F: Fn(NvimString, &mut LuaError) -> Object,
{
    let mut out_err = LuaError::default();
    let api_name = NvimString::new(name)?;

    let object = getter(api_name, &mut out_err);

    if out_err.is_err() {
        Err(Error::from(out_err))
    } else {
        Ok(object)
    }
}

/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
pub fn nvim_set_var(name: &str, value: Object) -> Result<(), Error> {
    let mut out_err = LuaError::default();
    let api_name = NvimString::new(name)?;

    unsafe {
        nvim::nvim_set_var(api_name, value, &mut out_err);
    }

    if out_err.is_err() {
        Err(Error::from(out_err))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// * If `name` can't be converted to a `NvimString`.
/// * If nvim set an error on the call.
///
pub fn nvim_set_vvar(name: &str, value: Object) -> Result<(), Error> {
    _set_var(name, value, |api_name, value, out_err| unsafe {
        nvim::nvim_set_vvar(api_name, value, out_err);
    })
}

fn _set_var<F>(name: &str, value: Object, setter: F) -> Result<(), Error>
where
    F: Fn(NvimString, Object, &mut LuaError),
{
    let mut out_err = LuaError::default();
    let api_name = NvimString::new(name)?;

    setter(api_name, value, &mut out_err);

    if out_err.is_err() {
        Err(Error::from(out_err))
    } else {
        Ok(())
    }
}

/// # Errors
///
/// * If `keys` can't be converted to a `NvimString`.
/// * If nvim set an error to `v:errmsg`.
///
pub fn nvim_feedkeys(keys: &str, mode: &str, escape_csi: bool) -> Result<(), Error> {
    let api_keys = NvimString::new(keys)?;
    let api_mode = NvimString::new(mode)?;

    unsafe {
        nvim::nvim_feedkeys(api_keys, api_mode, escape_csi);
    }

    match nvim_get_vvar("errmsg") {
        Ok(errmsg) => match errmsg.object_type() {
            ObjectType::kObjectTypeNil => Ok(()),
            ObjectType::kObjectTypeString if errmsg.as_string_unchecked().is_empty() => Ok(()),
            ObjectType::kObjectTypeString => {
                Err(Error::VErrMsg(errmsg.as_string_unchecked().clone()))
            }
            t => {
                eprintln!("Got unexpected v:errmsg object: {:?}", errmsg);
                Err(Error::ObjectError(nvim::object::Error::TypeError {
                    expected: ObjectType::kObjectTypeString,
                    actual: t,
                }))
            }
        },
        Err(_) => Ok(()),
    }
}

/// Gets the current buffer (number).
///
#[must_use]
pub fn nvim_get_current_buf() -> Buffer {
    unsafe { nvim::nvim_get_current_buf() }
}

/// The `Dictionary` returned from nvim  contains both a `mode` and `blocking` key.
///
#[must_use]
pub fn nvim_get_mode() -> Dictionary {
    unsafe { nvim::nvim_get_mode() }
}

//pub fn nvim_replace_termcodes(
//    string_to_convert: &str,
//    from_part: bool,
//    do_lt: bool,
//    special: bool,
//) -> String {
//    unsafe {
//        let api_string = cstr_to_string(string_to_convert.as_ptr() as *const c_char);

//        String::new(Cow::Owned(vim::nvim_replace_termcodes(
//            api_string, from_part, do_lt, special,
//        )))
//    }
//}

//pub fn nvim_exec(src: &str, output: bool) -> Result<String, Error> {
//    unsafe {
//        let api_src = cstr_to_string(src.as_ptr() as *const c_char);
//        let mut out_err = Error::default();

//        let vim_string = vim::nvim_exec(api_src, output, out_err.inner_mut());

//        if out_err.is_err() {
//            Err(out_err)
//        } else {
//            Ok(String::new(Cow::Owned(vim_string)))
//        }
//    }
//}

//pub fn nvim_set_hl(namespace_id: Integer, name: &str, val: Dictionary) -> Result<(), Error> {
//    unsafe {
//        let api_name = cstr_to_string(name.as_ptr() as *const c_char);
//        let mut out_err = Error::default();

//        vim::nvim_set_hl(namespace_id, api_name, val.inner(), out_err.inner_mut());

//        if out_err.is_err() {
//            Err(out_err)
//        } else {
//            Ok(())
//        }
//    }
//}

//pub fn nvim_get_namespaces() -> Dictionary {
//    unsafe { Dictionary::new(vim::nvim_get_namespaces()) }
//}

//pub fn nvim_create_namespace(name: &str) -> Integer {
//    unsafe {
//        let api_name = cstr_to_string(name.as_ptr() as *const c_char);

//        nvim::nvim_create_namespace(api_name)
//    }
//}
