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
// 初始化（单次调用，其他 provider 依赖此 provider）
// ============================================================================

final initProvider = FutureProvider<LianpkgConfig>((ref) async {
  final service = ref.read(lianpkgServiceProvider);
  return service.init();
});

// ============================================================================
// 配置（依赖 init 完成后再获取）
// ============================================================================

final configProvider = FutureProvider<LianpkgConfig>((ref) async {
  // 确保先完成初始化
  final initConfig = await ref.watch(initProvider.future);
  return initConfig;
});

// ============================================================================
// 壁纸扫描
// ============================================================================

final scanResultProvider = FutureProvider<ScanResult>((ref) async {
  // 确保先完成初始化
  await ref.watch(initProvider.future);
  final service = ref.read(lianpkgServiceProvider);
  return service.scan();
});

// ============================================================================
// 状态统计
// ============================================================================

final statusProvider = FutureProvider<StatusInfo>((ref) async {
  await ref.watch(initProvider.future);
  final service = ref.read(lianpkgServiceProvider);
  return service.getStatus();
});

final stateProvider = FutureProvider<StateData>((ref) async {
  await ref.watch(initProvider.future);
  final service = ref.read(lianpkgServiceProvider);
  return service.getState();
});

// ============================================================================
// 当前导航索引
// ============================================================================

final navigationIndexProvider = StateProvider<int>((ref) => 0);
