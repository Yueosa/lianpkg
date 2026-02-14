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
  final String? previewPath;
  final WallpaperType wallpaperType;
  final bool processed;
  final List<String> pkgFiles;
  final List<String> texFiles;
  final String workshopDir;

  const WallpaperInfo({
    required this.id,
    this.title,
    this.previewPath,
    required this.wallpaperType,
    required this.processed,
    required this.pkgFiles,
    required this.texFiles,
    required this.workshopDir,
  });

  factory WallpaperInfo.fromJson(Map<String, dynamic> json) {
    return WallpaperInfo(
      id: json['id'] as String? ?? '',
      title: json['title'] as String?,
      previewPath: json['preview_path'] as String?,
      wallpaperType: WallpaperType.fromString(
        json['wallpaper_type'] as String? ?? 'Raw',
      ),
      processed: json['processed'] as bool? ?? false,
      pkgFiles: (json['pkg_files'] as List<dynamic>?)?.cast<String>() ?? [],
      texFiles: (json['tex_files'] as List<dynamic>?)?.cast<String>() ?? [],
      workshopDir: json['workshop_dir'] as String? ?? '',
    );
  }
}

enum WallpaperType {
  pkg,
  raw,
  skipped;

  static WallpaperType fromString(String s) {
    return switch (s.toLowerCase()) {
      'pkg' || 'pkgtex' => WallpaperType.pkg,
      'raw' => WallpaperType.raw,
      _ => WallpaperType.skipped,
    };
  }

  String get label => switch (this) {
    WallpaperType.pkg => 'PKG',
    WallpaperType.raw => 'Raw',
    WallpaperType.skipped => 'Skipped',
  };
}

class ScanStats {
  final int total;
  final int pkg;
  final int raw;
  final int processed;
  final int unprocessed;

  const ScanStats({
    this.total = 0,
    this.pkg = 0,
    this.raw = 0,
    this.processed = 0,
    this.unprocessed = 0,
  });

  factory ScanStats.fromJson(Map<String, dynamic> json) {
    return ScanStats(
      total: json['total'] as int? ?? 0,
      pkg: json['pkg'] as int? ?? 0,
      raw: json['raw'] as int? ?? 0,
      processed: json['processed'] as int? ?? 0,
      unprocessed: json['unprocessed'] as int? ?? 0,
    );
  }
}
