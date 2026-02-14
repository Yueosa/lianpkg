/// dart:ffi 绑定层 — 加载 liblianpkg.so/.dll，封装 JSON 通信
library;

import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';

// ============================================================================
// C 函数签名
// ============================================================================

/// `extern "C" char* lianpkg_call(const char* json_input)`
typedef LianpkgCallNative = Pointer<Utf8> Function(Pointer<Utf8>);
typedef LianpkgCallDart = Pointer<Utf8> Function(Pointer<Utf8>);

/// `extern "C" void lianpkg_free_string(char* ptr)`
typedef LianpkgFreeNative = Void Function(Pointer<Utf8>);
typedef LianpkgFreeDart = void Function(Pointer<Utf8>);

// ============================================================================
// FFI Bridge
// ============================================================================

class FfiBridge {
  late final DynamicLibrary _lib;
  late final LianpkgCallDart _call;
  late final LianpkgFreeDart _free;

  static FfiBridge? _instance;

  FfiBridge._();

  /// 单例获取
  static FfiBridge get instance {
    _instance ??= FfiBridge._().._load();
    return _instance!;
  }

  void _load() {
    final libPath = _findLibrary();
    _lib = DynamicLibrary.open(libPath);

    _call = _lib
        .lookupFunction<LianpkgCallNative, LianpkgCallDart>('lianpkg_call');
    _free = _lib
        .lookupFunction<LianpkgFreeNative, LianpkgFreeDart>('lianpkg_free_string');
  }

  /// 查找动态库路径
  static String _findLibrary() {
    final String libName;
    if (Platform.isLinux) {
      libName = 'liblianpkg.so';
    } else if (Platform.isWindows) {
      libName = 'lianpkg.dll';
    } else {
      throw UnsupportedError('Unsupported platform: ${Platform.operatingSystem}');
    }

    // 搜索顺序：
    // 1. 可执行文件同目录（发布版）
    // 2. 可执行文件同目录/lib/（Linux bundle）
    // 3. 项目根目录 target/release/（开发模式）
    final exeDir = File(Platform.resolvedExecutable).parent.path;
    final candidates = [
      '$exeDir/$libName',
      '$exeDir/lib/$libName',
      // 开发模式：从 gui/ 向上找 target/release/
      '${File(Platform.resolvedExecutable).parent.parent.path}/target/release/$libName',
      // 直接在 lianpkg 项目根
      '${Directory.current.path}/../target/release/$libName',
      '${Directory.current.path}/target/release/$libName',
    ];

    for (final path in candidates) {
      if (File(path).existsSync()) {
        return path;
      }
    }

    // 最后尝试系统 PATH（Linux ld 搜索路径）
    return libName;
  }

  /// 同步调用 FFI（在当前线程执行）
  ///
  /// 适用于快速操作（init, config_get, progress 等）
  Map<String, dynamic> callSync(String action, [Map<String, dynamic>? params]) {
    final request = jsonEncode({
      'action': action,
      'params': params ?? {},
    });

    final inputPtr = request.toNativeUtf8();
    final resultPtr = _call(inputPtr);
    final resultJson = resultPtr.toDartString();

    // 释放内存
    calloc.free(inputPtr);
    _free(resultPtr);

    final result = jsonDecode(resultJson) as Map<String, dynamic>;
    return result;
  }

  /// 异步调用 FFI（在 Isolate 中执行）
  ///
  /// 适用于耗时操作（auto, scan, pkg_unpack, tex_convert 等）
  /// 不会阻塞 UI 线程
  Future<Map<String, dynamic>> callAsync(String action, [Map<String, dynamic>? params]) async {
    final request = jsonEncode({
      'action': action,
      'params': params ?? {},
    });

    // 在 Isolate 中执行 FFI 调用
    final resultJson = await Isolate.run(() {
      // Isolate 中需要重新加载动态库
      final libPath = _findLibrary();
      final lib = DynamicLibrary.open(libPath);

      final call = lib.lookupFunction<LianpkgCallNative, LianpkgCallDart>('lianpkg_call');
      final free = lib.lookupFunction<LianpkgFreeNative, LianpkgFreeDart>('lianpkg_free_string');

      final inputPtr = request.toNativeUtf8();
      final resultPtr = call(inputPtr);
      final result = resultPtr.toDartString();

      calloc.free(inputPtr);
      free(resultPtr);

      return result;
    });

    return jsonDecode(resultJson) as Map<String, dynamic>;
  }
}
