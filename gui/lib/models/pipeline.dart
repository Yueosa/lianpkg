/// 流水线输出 + 进度模型
library;

/// Auto 流水线返回的嵌套输出结构
class AutoOutput {
  final AutoPipelineStats stats;
  final CopyOutput? copyOutput;
  final UnpackOutput? pkgOutput;
  final ConvertOutput? texOutput;

  const AutoOutput({
    this.stats = const AutoPipelineStats(),
    this.copyOutput,
    this.pkgOutput,
    this.texOutput,
  });

  factory AutoOutput.fromJson(Map<String, dynamic> json) {
    return AutoOutput(
      stats: AutoPipelineStats.fromJson(
        json['stats'] as Map<String, dynamic>? ?? {},
      ),
      copyOutput: json['copy_output'] != null
          ? CopyOutput.fromJson(json['copy_output'] as Map<String, dynamic>)
          : null,
      pkgOutput: json['pkg_output'] != null
          ? UnpackOutput.fromJson(json['pkg_output'] as Map<String, dynamic>)
          : null,
      texOutput: json['tex_output'] != null
          ? ConvertOutput.fromJson(json['tex_output'] as Map<String, dynamic>)
          : null,
    );
  }

  /// 便捷：总耗时（秒）
  double get durationSecs => stats.elapsedMs / 1000.0;
}

class AutoPipelineStats {
  final int wallpapersProcessed;
  final int wallpapersSkipped;
  final int pkgsUnpacked;
  final int texsConverted;
  final int elapsedMs;

  const AutoPipelineStats({
    this.wallpapersProcessed = 0,
    this.wallpapersSkipped = 0,
    this.pkgsUnpacked = 0,
    this.texsConverted = 0,
    this.elapsedMs = 0,
  });

  factory AutoPipelineStats.fromJson(Map<String, dynamic> json) {
    return AutoPipelineStats(
      wallpapersProcessed: json['wallpapers_processed'] as int? ?? 0,
      wallpapersSkipped: json['wallpapers_skipped'] as int? ?? 0,
      pkgsUnpacked: json['pkgs_unpacked'] as int? ?? 0,
      texsConverted: json['texs_converted'] as int? ?? 0,
      elapsedMs: json['elapsed_ms'] as int? ?? 0,
    );
  }
}

class CopyOutput {
  final int rawCopied;
  final int pkgCopied;
  final int skipped;
  final int totalPkgFiles;

  const CopyOutput({
    this.rawCopied = 0,
    this.pkgCopied = 0,
    this.skipped = 0,
    this.totalPkgFiles = 0,
  });

  factory CopyOutput.fromJson(Map<String, dynamic> json) {
    final stats = json['stats'] as Map<String, dynamic>? ?? {};
    return CopyOutput(
      rawCopied: stats['raw_copied'] as int? ?? 0,
      pkgCopied: stats['pkg_copied'] as int? ?? 0,
      skipped: stats['skipped'] as int? ?? 0,
      totalPkgFiles: stats['total_pkg_files'] as int? ?? 0,
    );
  }

  int get copiedCount => rawCopied + pkgCopied;
}

class UnpackOutput {
  final int pkgProcessed;
  final int pkgSuccess;
  final int pkgFailed;
  final int totalFiles;
  final int texFiles;

  const UnpackOutput({
    this.pkgProcessed = 0,
    this.pkgSuccess = 0,
    this.pkgFailed = 0,
    this.totalFiles = 0,
    this.texFiles = 0,
  });

  factory UnpackOutput.fromJson(Map<String, dynamic> json) {
    final stats = json['stats'] as Map<String, dynamic>? ?? {};
    return UnpackOutput(
      pkgProcessed: stats['pkg_processed'] as int? ?? 0,
      pkgSuccess: stats['pkg_success'] as int? ?? 0,
      pkgFailed: stats['pkg_failed'] as int? ?? 0,
      totalFiles: stats['total_files'] as int? ?? 0,
      texFiles: stats['tex_files'] as int? ?? 0,
    );
  }
}

class ConvertOutput {
  final int texProcessed;
  final int texSuccess;
  final int texFailed;
  final int texSkipped;
  final int imageCount;
  final int videoCount;

  const ConvertOutput({
    this.texProcessed = 0,
    this.texSuccess = 0,
    this.texFailed = 0,
    this.texSkipped = 0,
    this.imageCount = 0,
    this.videoCount = 0,
  });

  factory ConvertOutput.fromJson(Map<String, dynamic> json) {
    final stats = json['stats'] as Map<String, dynamic>? ?? {};
    return ConvertOutput(
      texProcessed: stats['tex_processed'] as int? ?? 0,
      texSuccess: stats['tex_success'] as int? ?? 0,
      texFailed: stats['tex_failed'] as int? ?? 0,
      texSkipped: stats['tex_skipped'] as int? ?? 0,
      imageCount: stats['image_count'] as int? ?? 0,
      videoCount: stats['video_count'] as int? ?? 0,
    );
  }
}

class ProgressSnapshot {
  final bool running;
  final int percent;
  final String stage;
  final String message;
  final String? currentItem;

  const ProgressSnapshot({
    this.running = false,
    this.percent = 0,
    this.stage = '',
    this.message = '',
    this.currentItem,
  });

  factory ProgressSnapshot.fromJson(Map<String, dynamic> json) {
    return ProgressSnapshot(
      running: json['running'] as bool? ?? false,
      percent: json['percent'] as int? ?? 0,
      stage: json['stage'] as String? ?? '',
      message: json['message'] as String? ?? '',
      currentItem: json['current_item'] as String?,
    );
  }

  bool get isIdle => !running && percent == 0;
  bool get isDone => !running && percent >= 100;
}
