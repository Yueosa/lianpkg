/// TEX 预览模型
library;

class TexPreview {
  final String version;
  final String format;
  final int width;
  final int height;
  final int imageCount;
  final int mipmapCount;
  final bool isCompressed;
  final bool isVideo;
  final int dataSize;
  final String recommendedOutput;

  const TexPreview({
    required this.version,
    required this.format,
    required this.width,
    required this.height,
    required this.imageCount,
    required this.mipmapCount,
    required this.isCompressed,
    required this.isVideo,
    required this.dataSize,
    required this.recommendedOutput,
  });

  factory TexPreview.fromJson(Map<String, dynamic> json) {
    return TexPreview(
      version: json['version'] as String? ?? '',
      format: json['format'] as String? ?? '',
      width: json['width'] as int? ?? 0,
      height: json['height'] as int? ?? 0,
      imageCount: json['image_count'] as int? ?? 0,
      mipmapCount: json['mipmap_count'] as int? ?? 0,
      isCompressed: json['is_compressed'] as bool? ?? false,
      isVideo: json['is_video'] as bool? ?? false,
      dataSize: json['data_size'] as int? ?? 0,
      recommendedOutput: json['recommended_output'] as String? ?? '',
    );
  }

  /// 分辨率字符串
  String get resolution => '${width}x$height';

  /// 格式化数据大小
  String get formattedDataSize {
    if (dataSize < 1024) return '$dataSize B';
    if (dataSize < 1024 * 1024) {
      return '${(dataSize / 1024).toStringAsFixed(1)} KB';
    }
    return '${(dataSize / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  /// 类型标签
  String get typeLabel => isVideo ? 'Video' : 'Image';
}
