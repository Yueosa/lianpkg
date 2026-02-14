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
  final int copiedCount;
  final int skippedCount;
  final int errorCount;

  const CopyOutput({
    this.copiedCount = 0,
    this.skippedCount = 0,
    this.errorCount = 0,
  });

  factory CopyOutput.fromJson(Map<String, dynamic> json) {
    return CopyOutput(
      copiedCount: json['copied_count'] as int? ?? 0,
      skippedCount: json['skipped_count'] as int? ?? 0,
      errorCount: json['error_count'] as int? ?? 0,
    );
  }
}

class UnpackOutput {
  final int unpackedCount;
  final int errorCount;

  const UnpackOutput({this.unpackedCount = 0, this.errorCount = 0});

  factory UnpackOutput.fromJson(Map<String, dynamic> json) {
    return UnpackOutput(
      unpackedCount: json['unpacked_count'] as int? ?? 0,
      errorCount: json['error_count'] as int? ?? 0,
    );
  }
}

class ConvertOutput {
  final int imageCount;
  final int videoCount;
  final int errorCount;
  final int skippedCount;

  const ConvertOutput({
    this.imageCount = 0,
    this.videoCount = 0,
    this.errorCount = 0,
    this.skippedCount = 0,
  });

  factory ConvertOutput.fromJson(Map<String, dynamic> json) {
    return ConvertOutput(
      imageCount: json['image_count'] as int? ?? 0,
      videoCount: json['video_count'] as int? ?? 0,
      errorCount: json['error_count'] as int? ?? 0,
      skippedCount: json['skipped_count'] as int? ?? 0,
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
