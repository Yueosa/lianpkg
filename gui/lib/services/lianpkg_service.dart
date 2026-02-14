/// 业务层 API 封装 — 将 FFI JSON 响应转换为类型安全的 Dart 模型
library;

import 'dart:async';

import '../models/config.dart';
import '../models/pipeline.dart';
import '../models/state.dart';
import '../models/wallpaper.dart';
import 'ffi_bridge.dart';

class LianpkgService {
  final FfiBridge _ffi = FfiBridge.instance;

  // ============================================================================
  // 初始化
  // ============================================================================

  /// 初始化 lianpkg 上下文，返回运行时配置
  Future<LianpkgConfig> init({String? configDir}) async {
    final result = await _ffi.callAsync('init', {?'config_dir': configDir});
    _checkError(result);
    return LianpkgConfig.fromJson(result['data'] as Map<String, dynamic>);
  }

  // ============================================================================
  // 壁纸浏览
  // ============================================================================

  /// 扫描 Workshop 壁纸
  Future<ScanResult> scan({String? workshopPath}) async {
    final result = await _ffi.callAsync('scan', {
      if (workshopPath != null) 'workshop_path': workshopPath,
    });
    _checkError(result);
    return ScanResult.fromJson(result['data'] as Map<String, dynamic>);
  }

  // ============================================================================
  // 流水线
  // ============================================================================

  /// 运行 Auto 流水线
  Future<AutoOutput> runAuto({
    List<String>? wallpaperIds,
    bool noRaw = false,
    bool noTex = false,
    bool noCleanUnpacked = false,
    bool noIncremental = false,
  }) async {
    final result = await _ffi.callAsync('auto', {
      if (wallpaperIds != null) 'wallpaper_ids': wallpaperIds,
      'no_raw': noRaw,
      'no_tex': noTex,
      'no_clean_unpacked': noCleanUnpacked,
      'no_incremental': noIncremental,
    });
    _checkError(result);
    return AutoOutput.fromJson(result['data'] as Map<String, dynamic>);
  }

  /// 轮询 Auto 进度（同步，不阻塞 UI）
  ProgressSnapshot pollProgress() {
    final result = _ffi.callSync('progress');
    _checkError(result);
    return ProgressSnapshot.fromJson(result['data'] as Map<String, dynamic>);
  }

  // ============================================================================
  // PKG 操作
  // ============================================================================

  /// 解包 PKG 文件
  Future<Map<String, dynamic>> unpackPkg({
    required List<PkgSourceDto> sources,
    required String output,
  }) async {
    final result = await _ffi.callAsync('pkg_unpack', {
      'sources': sources.map((s) => s.toJson()).toList(),
      'output': output,
    });
    _checkError(result);
    return result['data'] as Map<String, dynamic>;
  }

  /// 预览 PKG 文件元数据
  Future<Map<String, dynamic>> previewPkg(String path) async {
    final result = await _ffi.callAsync('pkg_preview', {'path': path});
    _checkError(result);
    return result['data'] as Map<String, dynamic>;
  }

  // ============================================================================
  // TEX 操作
  // ============================================================================

  /// 批量转换 TEX → PNG
  Future<Map<String, dynamic>> convertTex({
    required String input,
    String? output,
  }) async {
    final result = await _ffi.callAsync('tex_convert', {
      'input': input,
      if (output != null) 'output': output,
    });
    _checkError(result);
    return result['data'] as Map<String, dynamic>;
  }

  /// 预览 TEX 文件元数据
  Future<Map<String, dynamic>> previewTex(String path) async {
    final result = await _ffi.callAsync('tex_preview', {'path': path});
    _checkError(result);
    return result['data'] as Map<String, dynamic>;
  }

  // ============================================================================
  // 配置
  // ============================================================================

  /// 获取当前配置
  Future<LianpkgConfig> getConfig() async {
    final result = await _ffi.callAsync('config_get');
    _checkError(result);
    return LianpkgConfig.fromJson(result['data'] as Map<String, dynamic>);
  }

  /// 设置配置项
  Future<void> setConfig(String key, String value) async {
    final result = await _ffi.callAsync('config_set', {
      'key': key,
      'value': value,
    });
    _checkError(result);
  }

  /// 重置配置为默认值
  Future<void> resetConfig() async {
    final result = await _ffi.callAsync('config_reset');
    _checkError(result);
  }

  // ============================================================================
  // 状态
  // ============================================================================

  /// 获取处理状态
  Future<StateData> getState() async {
    final result = await _ffi.callAsync('state_get');
    _checkError(result);
    return StateData.fromJson(result['data'] as Map<String, dynamic>);
  }

  /// 清空处理状态
  Future<void> clearState() async {
    final result = await _ffi.callAsync('state_clear');
    _checkError(result);
  }

  /// 获取综合状态（统计 + 磁盘估算）
  Future<StatusInfo> getStatus() async {
    final result = await _ffi.callAsync('status');
    _checkError(result);
    return StatusInfo.fromJson(result['data'] as Map<String, dynamic>);
  }

  // ============================================================================
  // 辅助
  // ============================================================================

  void _checkError(Map<String, dynamic> result) {
    if (result['success'] != true) {
      throw LianpkgException(result['error']?.toString() ?? 'Unknown error');
    }
  }
}

/// PKG 解包源参数
class PkgSourceDto {
  final String wallpaperId;
  final List<String> pkgPaths;

  const PkgSourceDto({required this.wallpaperId, required this.pkgPaths});

  Map<String, dynamic> toJson() => {
    'wallpaper_id': wallpaperId,
    'pkg_paths': pkgPaths,
  };
}

/// lianpkg 通信异常
class LianpkgException implements Exception {
  final String message;
  const LianpkgException(this.message);

  @override
  String toString() => 'LianpkgException: $message';
}
