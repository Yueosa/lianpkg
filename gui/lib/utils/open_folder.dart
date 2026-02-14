/// 打开文件资源管理器的工具函数
library;

import 'dart:io';

/// 在系统文件资源管理器中打开指定目录
///
/// Linux: xdg-open, Windows: explorer
Future<bool> openFolder(String path) async {
  final dir = Directory(path);
  if (!dir.existsSync()) return false;

  try {
    if (Platform.isLinux) {
      await Process.run('xdg-open', [path]);
    } else if (Platform.isWindows) {
      await Process.run('explorer', [path]);
    } else {
      return false;
    }
    return true;
  } catch (_) {
    return false;
  }
}
