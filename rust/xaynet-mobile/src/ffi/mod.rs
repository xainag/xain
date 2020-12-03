#![allow(unused_unsafe)]

mod participant;
pub use participant::*;

mod settings;
pub use settings::*;

pub use ffi_support::{ByteBuffer, FfiStr};
use std::os::raw::c_int;

/// Destroy the given `ByteBuffer` and free its memory. This function must only be
/// called on `ByteBuffer`s that have been created on the Rust side of the FFI. If you
/// have created a `ByteBuffer` on the other side of the FFI, do not use this function,
/// use `free()` instead.
///
/// # Return value
///
/// - [`OK`] on success
/// - [`ERR_NULLPTR`] if `buf` is NULL
///
/// # Safety
///
/// 1. When calling this method, you have to ensure that *either* the pointer is NULL
/// *or* all of the following is true:
///  - The pointer must be properly [aligned].
///  - It must be "dereferencable" in the sense defined in the [`::std::ptr`] module
///    documentation.
/// 2. After destroying the `ByteBuffer` the pointer becomes invalid and must not be
///    used.
/// 3. Calling this function on a `ByteBuffer` that has not been created on the Rust
///    side of the FFI is UB.
///
/// [`::std::ptr`]: https://doc.rust-lang.org/std/ptr/index.html#safety
/// [aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[no_mangle]
pub unsafe extern "C" fn xaynet_ffi_byte_buffer_destroy(
    // Note that we use a *const instead of a *mut here. The reason is
    // that the functions that create byte buffers return *const
    // pointers. Taking a *mut here would trigger a
    // -Wdiscarded-qualifiers warning from C. Forcing users to use
    // *const pointers brings some safety, and casting back to *mut
    // here is no big deal since the pointer becomes invalid afterward
    // anyway.
    buf: *const ByteBuffer,
) -> c_int {
    if buf.is_null() {
        return ERR_NULLPTR;
    }
    Box::from_raw(buf as *mut ByteBuffer).destroy();
    OK
}

/// Initialize the crypto library. This method must be called before instantiating a
/// participant with [`xaynet_ffi_participant_new()`] or before generating new keys with
/// [`xaynet_ffi_generate_key_pair()`].
///
/// # Return value
///
/// - [`OK`] if the initialization succeeded
/// - -[`ERR_CRYPTO_INIT`] if the initialization failed
///
/// [`xaynet_ffi_participant_new()`]: xaynet_ffi_participant_new
/// [`xaynet_ffi_generate_key_pair()`]: xaynet_ffi_generate_key_pair
///
/// # Safety
///
/// This function is safe to call
#[no_mangle]
pub unsafe extern "C" fn xaynet_ffi_crypto_init() -> c_int {
    if sodiumoxide::init().is_err() {
        ERR_CRYPTO_INIT
    } else {
        OK
    }
}

/// Return value upon success
pub const OK: c_int = 0;
/// NULL pointer argument
pub const ERR_NULLPTR: c_int = 1;
/// Invalid mask configuration: invalid group type
pub const ERR_MASKCONFIG_GROUPTYPE: c_int = 2;
/// Invalid mask configuration: invalid data type
pub const ERR_MASKCONFIG_DATATYPE: c_int = 3;
/// Invalid mask configuration: invalid bound type
pub const ERR_MASKCONFIG_BOUNDTYPE: c_int = 4;
/// Invalid mask configuration: invalid model type
pub const ERR_MASKCONFIG_MODELTYPE: c_int = 5;
/// Invalid coordinator URL
pub const ERR_INVALID_URL: c_int = 6;
/// Invalid settings: coordinator URL is not set
pub const ERR_SETTINGS_URL: c_int = 7;
/// Invalid settings: mask configuration is not set
pub const ERR_SETTINGS_MASKCONFIG: c_int = 8;
/// Invalid settings: signing keys are not set
pub const ERR_SETTINGS_KEYS: c_int = 9;
/// Failed to set the local model: invalid model
pub const ERR_SETMODEL_MODEL: c_int = 10;
/// Failed to set the local model: data type
pub const ERR_SETMODEL_DATATYPE: c_int = 11;
/// Failed to initialized the crypto library
pub const ERR_CRYPTO_INIT: c_int = 12;
/// Invalid secret signing key
pub const ERR_CRYPTO_SECRET_KEY: c_int = 13;
/// Invalid public signing key
pub const ERR_CRYPTO_PUBLIC_KEY: c_int = 14;
