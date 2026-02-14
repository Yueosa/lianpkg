/// 处理状态 + 状态统计模型
library;

class StateData {
  final Map<String, ProcessedEntry> processed;
  final String? lastRun;

  const StateData({this.processed = const {}, this.lastRun});

  factory StateData.fromJson(Map<String, dynamic> json) {
    final processedMap = <String, ProcessedEntry>{};
    final raw = json['processed'] as Map<String, dynamic>? ?? {};
    for (final entry in raw.entries) {
      processedMap[entry.key] = ProcessedEntry.fromJson(
        entry.value as Map<String, dynamic>,
      );
    }
    return StateData(
      processed: processedMap,
      lastRun: json['last_run'] as String?,
    );
  }
}

class ProcessedEntry {
  final String? title;
  final String processType;
  final String processedAt;
  final String? outputPath;

  const ProcessedEntry({
    this.title,
    required this.processType,
    required this.processedAt,
    this.outputPath,
  });

  factory ProcessedEntry.fromJson(Map<String, dynamic> json) {
    return ProcessedEntry(
      title: json['title'] as String?,
      processType: json['process_type'] as String? ?? 'Skipped',
      processedAt: json['processed_at'] as String? ?? '',
      outputPath: json['output_path'] as String?,
    );
  }
}

class StatusInfo {
  final int totalProcessed;
  final int pkgCount;
  final int rawCount;
  final int skippedCount;
  final String? lastRun;
  final DiskEstimate diskEstimate;

  const StatusInfo({
    this.totalProcessed = 0,
    this.pkgCount = 0,
    this.rawCount = 0,
    this.skippedCount = 0,
    this.lastRun,
    this.diskEstimate = const DiskEstimate(),
  });

  factory StatusInfo.fromJson(Map<String, dynamic> json) {
    return StatusInfo(
      totalProcessed: json['total_processed'] as int? ?? 0,
      pkgCount: json['pkg_count'] as int? ?? 0,
      rawCount: json['raw_count'] as int? ?? 0,
      skippedCount: json['skipped_count'] as int? ?? 0,
      lastRun: json['last_run'] as String?,
      diskEstimate: DiskEstimate.fromJson(
        json['disk_estimate'] as Map<String, dynamic>? ?? {},
      ),
    );
  }
}

/// 磁盘用量估算
class DiskEstimate {
  final int pkgSize;
  final int rawSize;
  final int pkgCount;
  final int rawCount;
  final int estimatedUnpacked;
  final int estimatedConverted;
  final int estimatedPeak;
  final int estimatedFinal;
  final int? availableSpace;
  final bool spaceSufficient;

  const DiskEstimate({
    this.pkgSize = 0,
    this.rawSize = 0,
    this.pkgCount = 0,
    this.rawCount = 0,
    this.estimatedUnpacked = 0,
    this.estimatedConverted = 0,
    this.estimatedPeak = 0,
    this.estimatedFinal = 0,
    this.availableSpace,
    this.spaceSufficient = true,
  });

  factory DiskEstimate.fromJson(Map<String, dynamic> json) {
    return DiskEstimate(
      pkgSize: json['pkg_size'] as int? ?? 0,
      rawSize: json['raw_size'] as int? ?? 0,
      pkgCount: json['pkg_count'] as int? ?? 0,
      rawCount: json['raw_count'] as int? ?? 0,
      estimatedUnpacked: json['estimated_unpacked'] as int? ?? 0,
      estimatedConverted: json['estimated_converted'] as int? ?? 0,
      estimatedPeak: json['estimated_peak'] as int? ?? 0,
      estimatedFinal: json['estimated_final'] as int? ?? 0,
      availableSpace: json['available_space'] as int?,
      spaceSufficient: json['space_sufficient'] as bool? ?? true,
    );
  }

  /// 格式化字节为可读字符串
  static String formatBytes(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
    }
    return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(1)} GB';
  }
}
