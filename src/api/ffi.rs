use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use crate::core::config::Config;
use crate::api::{native, types::*};

// Helper to parse C string to Rust string
unsafe fn parse_c_string(s: *const c_char) -> Result<String, String> {
    if s.is_null() {
        return Err("Null pointer received".to_string());
    }
    // SAFETY: The caller must ensure that `s` is a valid pointer to a null-terminated C string.
    unsafe {
        CStr::from_ptr(s)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| format!("Invalid UTF-8: {}", e))
    }
}

// Helper to return JSON string to C
fn return_json<T: serde::Serialize>(result: &T) -> *mut c_char {
    let json = serde_json::to_string(result).unwrap_or_else(|_| "{\"status\":\"Error\",\"message\":\"JSON serialization failed\",\"data\":null}".to_string());
    CString::new(json).unwrap().into_raw()
}

// Helper to return error JSON
fn return_error(msg: &str) -> *mut c_char {
    let result = OperationResult::<()> {
        status: StatusCode::Error,
        message: msg.to_string(),
        data: None,
    };
    return_json(&result)
}

#[unsafe(no_mangle)]
pub extern "C" fn lianpkg_free_string(s: *mut c_char) {
    if s.is_null() { return; }
    unsafe {
        let _ = CString::from_raw(s);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lianpkg_run_wallpaper(config_json: *const c_char) -> *mut c_char {
    let config_str = unsafe {
        match parse_c_string(config_json) {
            Ok(s) => s,
            Err(e) => return return_error(&e),
        }
    };

    let config: Config = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => return return_error(&format!("Invalid Config JSON: {}", e)),
    };

    match native::run_wallpaper(&config) {
        Ok(data) => return_json(&OperationResult {
            status: StatusCode::Success,
            message: "Wallpaper extraction successful".to_string(),
            data: Some(data),
        }),
        Err(e) => return_error(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lianpkg_run_pkg(config_json: *const c_char) -> *mut c_char {
    let config_str = unsafe {
        match parse_c_string(config_json) {
            Ok(s) => s,
            Err(e) => return return_error(&e),
        }
    };

    let config: Config = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => return return_error(&format!("Invalid Config JSON: {}", e)),
    };

    match native::run_pkg(&config) {
        Ok(data) => return_json(&OperationResult {
            status: StatusCode::Success,
            message: "PKG unpack successful".to_string(),
            data: Some(data),
        }),
        Err(e) => return_error(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lianpkg_run_tex(config_json: *const c_char) -> *mut c_char {
    let config_str = unsafe {
        match parse_c_string(config_json) {
            Ok(s) => s,
            Err(e) => return return_error(&e),
        }
    };

    let config: Config = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => return return_error(&format!("Invalid Config JSON: {}", e)),
    };

    match native::run_tex(&config) {
        Ok(data) => return_json(&OperationResult {
            status: StatusCode::Success,
            message: "TEX conversion successful".to_string(),
            data: Some(data),
        }),
        Err(e) => return_error(&e),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn lianpkg_run_auto(config_json: *const c_char) -> *mut c_char {
    let config_str = unsafe {
        match parse_c_string(config_json) {
            Ok(s) => s,
            Err(e) => return return_error(&e),
        }
    };

    let config: Config = match serde_json::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => return return_error(&format!("Invalid Config JSON: {}", e)),
    };

    match native::run_auto(&config) {
        Ok(data) => return_json(&OperationResult {
            status: StatusCode::Success,
            message: "Auto mode successful".to_string(),
            data: Some(data),
        }),
        Err(e) => return_error(&e),
    }
}
