/// 流水线输出 + 进度模型
library;

class AutoOutput {
  final int wallpapersProcessed;
  final int pkgUnpacked;
  final int texConverted;
  final int imagesProduced;
  final int videosProduced;
  final int skipped;
  final int errors;
  final double durationSecs;

  const AutoOutput({
    this.wallpapersProcessed = 0,
    this.pkgUnpacked = 0,
    this.texConverted = 0,
    this.imagesProduced = 0,
    this.videosProduced = 0,
    this.skipped = 0,
    this.errors = 0,
    this.durationSecs = 0,
  });

  factory AutoOutput.fromJson(Map<String, dynamic> json) {
    return AutoOutput(
      wallpapersProcessed: json['wallpapers_processed'] as int? ?? 0,
      pkgUnpacked: json['pkg_unpacked'] as int? ?? 0,
      texConverted: json['tex_converted'] as int? ?? 0,
      imagesProduced: json['images_produced'] as int? ?? 0,
      videosProduced: json['videos_produced'] as int? ?? 0,
      skipped: json['skipped'] as int? ?? 0,
      errors: json['errors'] as int? ?? 0,
      durationSecs: (json['duration_secs'] as num?)?.toDouble() ?? 0,
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
