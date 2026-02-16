/// 打开文件资源管理器的工具函数
library;

import 'dart:io';

/// 在系统文件资源管理器中打开指定目录
///
/// 目录不存在时自动递归创建。
/// Linux: xdg-open, Windows: explorer
Future<bool> openFolder(String path) async {
  try {
    final dir = Directory(path);
    if (!dir.existsSync()) {
      dir.createSync(recursive: true);
    }

    if (Platform.isLinux) {
      await Process.run('xdg-open', [path]);
    } else if (Platform.isWindows) {
      final winPath = path.replaceAll('/', '\\');
      await Process.run('cmd', ['/c', 'start', '', winPath]);
    } else {
      return false;
    }
    return true;
  } catch (_) {
    return false;
  }
}
