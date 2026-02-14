//! FFI 接口 - 为 Flutter/Dart 提供 C 兼容的外部函数
//!
//! ## 通信协议
//!
//! 使用 JSON 进行序列化通信：
//!
//! ### 请求格式
//! ```json
//! {
//!   "action": "init" | "scan" | "auto" | "pkg_unpack" | "tex_convert" | "config_get" | "config_set" | "status",
//!   "params": { /* action 特定参数 */ }
//! }
//! ```
//!
//! ### 响应格式
//! ```json
//! {
//!   "success": true | false,
//!   "data": { /* 成功时的结果数据 */ } | null,
//!   "error": "错误消息" | null
//! }
//! ```
//!
//! ## 使用流程（Flutter/Dart 侧）
//!
//! ```dart
//! import 'dart:ffi';
//! import 'dart:convert';
//! import 'package:ffi/ffi.dart';
//!
//! // 1. 加载动态库
//! final DynamicLibrary lib = DynamicLibrary.open('liblianpkg.so'); // Linux
//! // final DynamicLibrary lib = DynamicLibrary.open('lianpkg.dll'); // Windows
//!
//! // 2. 定义函数签名
//! typedef LianpkgCallNative = Pointer<Utf8> Function(Pointer<Utf8>);
//! typedef LianpkgCallDart = Pointer<Utf8> Function(Pointer<Utf8>);
//! typedef LianpkgFreeNative = Void Function(Pointer<Utf8>);
//! typedef LianpkgFreeDart = void Function(Pointer<Utf8>);
//!
//! final lianpkgCall = lib.lookupFunction<LianpkgCallNative, LianpkgCallDart>('lianpkg_call');
//! final lianpkgFree = lib.lookupFunction<LianpkgFreeNative, LianpkgFreeDart>('lianpkg_free_string');
//!
//! // 3. 调用示例
//! String callLianpkg(Map<String, dynamic> request) {
//!   final requestJson = jsonEncode(request);
//!   final requestPtr = requestJson.toNativeUtf8();
//!   
//!   final responsePtr = lianpkgCall(requestPtr);
//!   final responseJson = responsePtr.toDartString();
//!   
//!   malloc.free(requestPtr);
//!   lianpkgFree(responsePtr);
//!   
//!   return responseJson;
//! }
//!
//! // 4. 使用
//! final result = callLianpkg({
//!   'action': 'init',
//!   'params': {'config_dir': null}
//! });
//! final response = jsonDecode(result);
//! if (response['success']) {
//!   print('Context: ${response['data']}');
//! } else {
//!   print('Error: ${response['error']}');
//! }
//! ```

mod types;
mod dispatch;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::panic;

use types::{FfiRequest, FfiResponse};

// ============================================================================
// C 外部接口
// ============================================================================

/// 主入口函数：接收 JSON 请求，返回 JSON 响应
///
/// # Safety
///
/// - `json_input` 必须是有效的 null-terminated C 字符串指针
/// - 调用方必须在使用完返回的指针后调用 `lianpkg_free_string` 释放内存
///
/// # 示例
///
/// ```c
/// const char* request = "{\"action\":\"init\",\"params\":{}}";
/// char* response = lianpkg_call(request);
/// printf("Response: %s\n", response);
/// lianpkg_free_string(response);
/// ```
#[no_mangle]
pub unsafe extern "C" fn lianpkg_call(json_input: *const c_char) -> *mut c_char {
    // 捕获所有 panic，防止跨 FFI 边界传播
    let result = panic::catch_unwind(|| {
        // 1. 将 C 字符串转为 Rust String
        if json_input.is_null() {
            return create_error_response("Input is null");
        }

        let c_str = unsafe { CStr::from_ptr(json_input) };
        let input_str = match c_str.to_str() {
            Ok(s) => s,
            Err(e) => return create_error_response(&format!("Invalid UTF-8: {}", e)),
        };

        // 2. 解析请求
        let request: FfiRequest = match serde_json::from_str(input_str) {
            Ok(req) => req,
            Err(e) => return create_error_response(&format!("Invalid JSON request: {}", e)),
        };

        // 3. 分发处理
        let response = dispatch::dispatch(request);

        // 4. 序列化响应
        match serde_json::to_string(&response) {
            Ok(json) => json,
            Err(e) => create_error_response(&format!("Failed to serialize response: {}", e)),
        }
    });

    // 5. 处理 panic
    let response_json = match result {
        Ok(json) => json,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                format!("Panic: {}", s)
            } else if let Some(s) = e.downcast_ref::<String>() {
                format!("Panic: {}", s)
            } else {
                "Panic: unknown error".to_string()
            };
            create_error_response(&msg)
        }
    };

    // 6. 转换为 C 字符串并返回（调用方负责释放）
    match CString::new(response_json) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => {
            // 极端情况：响应包含 null 字节
            let fallback = create_error_response("Response contains null byte");
            CString::new(fallback).unwrap().into_raw()
        }
    }
}

/// 释放由 `lianpkg_call` 返回的字符串
///
/// # Safety
///
/// - `s` 必须是由 `lianpkg_call` 返回的指针
/// - 每个指针只能释放一次
///
/// # 示例
///
/// ```c
/// char* response = lianpkg_call(request);
/// // ... 使用 response ...
/// lianpkg_free_string(response);
/// ```
#[no_mangle]
pub unsafe extern "C" fn lianpkg_free_string(s: *mut c_char) {
    if !s.is_null() {
        // 将原始指针重新转换为 CString，随后自动释放
        unsafe { drop(CString::from_raw(s)) };
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 创建错误响应的 JSON 字符串
fn create_error_response(message: &str) -> String {
    let response = FfiResponse::error(message);
    serde_json::to_string(&response).unwrap_or_else(|_| {
        // 如果连序列化都失败，返回硬编码的错误
        r#"{"success":false,"error":"Fatal: failed to create error response"}"#.to_string()
    })
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_ffi_init() {
        let request = r#"{"action":"init","params":{}}"#;
        let c_request = CString::new(request).unwrap();
        
        unsafe {
            let response_ptr = lianpkg_call(c_request.as_ptr());
            assert!(!response_ptr.is_null());
            
            let response_cstr = CStr::from_ptr(response_ptr);
            let response_str = response_cstr.to_str().unwrap();
            
            let response: FfiResponse = serde_json::from_str(response_str).unwrap();
            // init 可能失败（测试环境没有配置文件），但至少应该有响应
            assert!(response.success || response.error.is_some());
            
            lianpkg_free_string(response_ptr);
        }
    }

    #[test]
    fn test_ffi_invalid_json() {
        let request = r#"not a json"#;
        let c_request = CString::new(request).unwrap();
        
        unsafe {
            let response_ptr = lianpkg_call(c_request.as_ptr());
            let response_cstr = CStr::from_ptr(response_ptr);
            let response_str = response_cstr.to_str().unwrap();
            
            let response: FfiResponse = serde_json::from_str(response_str).unwrap();
            assert!(!response.success);
            assert!(response.error.is_some());
            
            lianpkg_free_string(response_ptr);
        }
    }

    #[test]
    fn test_ffi_unknown_action() {
        let request = r#"{"action":"unknown","params":{}}"#;
        let c_request = CString::new(request).unwrap();
        
        unsafe {
            let response_ptr = lianpkg_call(c_request.as_ptr());
            let response_cstr = CStr::from_ptr(response_ptr);
            let response_str = response_cstr.to_str().unwrap();
            
            let response: FfiResponse = serde_json::from_str(response_str).unwrap();
            assert!(!response.success);
            assert!(response.error.unwrap().contains("Unknown action"));
            
            lianpkg_free_string(response_ptr);
        }
    }
}
