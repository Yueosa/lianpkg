/// 壁纸扫描结果模型
library;

class ScanResult {
  final List<WallpaperInfo> wallpapers;
  final ScanStats stats;

  const ScanResult({required this.wallpapers, required this.stats});

  factory ScanResult.fromJson(Map<String, dynamic> json) {
    final wallpaperList =
        (json['wallpapers'] as List<dynamic>?)
            ?.map((e) => WallpaperInfo.fromJson(e as Map<String, dynamic>))
            .toList() ??
        [];
    return ScanResult(
      wallpapers: wallpaperList,
      stats: ScanStats.fromJson(json['stats'] as Map<String, dynamic>? ?? {}),
    );
  }
}

class WallpaperInfo {
  final String id;
  final String? title;
  final String? wallpaperType; // Rust 原始类型: "scene"/"video"/"web" 等
  final String? previewPath;
  final bool hasPkg;
  final bool isProcessed;
  final List<String> pkgFiles;
  final String folderPath;

  const WallpaperInfo({
    required this.id,
    this.title,
    this.wallpaperType,
    this.previewPath,
    required this.hasPkg,
    required this.isProcessed,
    required this.pkgFiles,
    required this.folderPath,
  });

  factory WallpaperInfo.fromJson(Map<String, dynamic> json) {
    return WallpaperInfo(
      id: json['wallpaper_id'] as String? ?? '',
      title: json['title'] as String?,
      wallpaperType: json['wallpaper_type'] as String?,
      previewPath: json['preview_path'] as String?,
      hasPkg: json['has_pkg'] as bool? ?? false,
      isProcessed: json['is_processed'] as bool? ?? false,
      pkgFiles: (json['pkg_files'] as List<dynamic>?)?.cast<String>() ?? [],
      folderPath: json['folder_path'] as String? ?? '',
    );
  }

  /// 基于 has_pkg 判断壁纸分类
  WallpaperCategory get category =>
      hasPkg ? WallpaperCategory.pkg : WallpaperCategory.raw;
}

/// 壁纸分类（基于是否含 PKG 文件）
enum WallpaperCategory {
  pkg,
  raw;

  String get label => switch (this) {
    WallpaperCategory.pkg => 'PKG',
    WallpaperCategory.raw => 'Raw',
  };
}

class ScanStats {
  final int totalCount;
  final int pkgCount;
  final int rawCount;

  const ScanStats({this.totalCount = 0, this.pkgCount = 0, this.rawCount = 0});

  factory ScanStats.fromJson(Map<String, dynamic> json) {
    return ScanStats(
      totalCount: json['total_count'] as int? ?? 0,
      pkgCount: json['pkg_count'] as int? ?? 0,
      rawCount: json['raw_count'] as int? ?? 0,
    );
  }
}
