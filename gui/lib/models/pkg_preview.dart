/// PKG 预览模型
library;

class PkgPreview {
  final String version;
  final int fileCount;
  final List<PkgFileEntry> files;
  final int texCount;

  const PkgPreview({
    required this.version,
    required this.fileCount,
    required this.files,
    required this.texCount,
  });

  factory PkgPreview.fromJson(Map<String, dynamic> json) {
    return PkgPreview(
      version: json['version'] as String? ?? '',
      fileCount: json['file_count'] as int? ?? 0,
      files:
          (json['files'] as List<dynamic>?)
              ?.map((e) => PkgFileEntry.fromJson(e as Map<String, dynamic>))
              .toList() ??
          [],
      texCount: json['tex_count'] as int? ?? 0,
    );
  }
}

class PkgFileEntry {
  final String name;
  final int size;
  final bool isTex;

  const PkgFileEntry({
    required this.name,
    required this.size,
    required this.isTex,
  });

  factory PkgFileEntry.fromJson(Map<String, dynamic> json) {
    return PkgFileEntry(
      name: json['name'] as String? ?? '',
      size: json['size'] as int? ?? 0,
      isTex: json['is_tex'] as bool? ?? false,
    );
  }

  /// 格式化文件大小
  String get formattedSize {
    if (size < 1024) return '$size B';
    if (size < 1024 * 1024) return '${(size / 1024).toStringAsFixed(1)} KB';
    return '${(size / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
}
