/// Riverpod providers — 全局状态管理
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/config.dart';
import '../models/state.dart';
import '../models/wallpaper.dart';
import '../services/lianpkg_service.dart';

// ============================================================================
// Service provider
// ============================================================================

final lianpkgServiceProvider = Provider<LianpkgService>((ref) {
  return LianpkgService();
});

// ============================================================================
// 配置
// ============================================================================

final configProvider = FutureProvider<LianpkgConfig>((ref) async {
  final service = ref.read(lianpkgServiceProvider);
  return service.getConfig();
});

// ============================================================================
// 壁纸扫描
// ============================================================================

final scanResultProvider = FutureProvider<ScanResult>((ref) async {
  final service = ref.read(lianpkgServiceProvider);
  // 先确保已初始化
  await service.init();
  return service.scan();
});

// ============================================================================
// 状态统计
// ============================================================================

final statusProvider = FutureProvider<StatusInfo>((ref) async {
  final service = ref.read(lianpkgServiceProvider);
  return service.getStatus();
});

final stateProvider = FutureProvider<StateData>((ref) async {
  final service = ref.read(lianpkgServiceProvider);
  return service.getState();
});

// ============================================================================
// 当前导航索引
// ============================================================================

final navigationIndexProvider = StateProvider<int>((ref) => 0);
