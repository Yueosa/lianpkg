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
  final int totalWallpapers;
  final int totalProcessed;
  final int processedPkg;
  final int processedRaw;
  final int processedSkipped;
  final int pendingTotal;
  final int pendingPkg;
  final int pendingRaw;
  final int pendingPkgSize;
  final String? lastRun;
  final DiskUsage diskUsage;

  const StatusInfo({
    this.totalWallpapers = 0,
    this.totalProcessed = 0,
    this.processedPkg = 0,
    this.processedRaw = 0,
    this.processedSkipped = 0,
    this.pendingTotal = 0,
    this.pendingPkg = 0,
    this.pendingRaw = 0,
    this.pendingPkgSize = 0,
    this.lastRun,
    this.diskUsage = const DiskUsage(),
  });

  factory StatusInfo.fromJson(Map<String, dynamic> json) {
    return StatusInfo(
      totalWallpapers: json['total_wallpapers'] as int? ?? 0,
      totalProcessed: json['total_processed'] as int? ?? 0,
      processedPkg: json['processed_pkg'] as int? ?? 0,
      processedRaw: json['processed_raw'] as int? ?? 0,
      processedSkipped: json['processed_skipped'] as int? ?? 0,
      pendingTotal: json['pending_total'] as int? ?? 0,
      pendingPkg: json['pending_pkg'] as int? ?? 0,
      pendingRaw: json['pending_raw'] as int? ?? 0,
      pendingPkgSize: json['pending_pkg_size'] as int? ?? 0,
      lastRun: json['last_run'] as String?,
      diskUsage: DiskUsage.fromJson(
        json['disk_usage'] as Map<String, dynamic>? ?? {},
      ),
    );
  }
}

/// 实际磁盘占用
class DiskUsage {
  final int rawOutputSize;
  final int unpackedOutputSize;
  final int convertedOutputSize;
  final int? availableSpace;

  const DiskUsage({
    this.rawOutputSize = 0,
    this.unpackedOutputSize = 0,
    this.convertedOutputSize = 0,
    this.availableSpace,
  });

  factory DiskUsage.fromJson(Map<String, dynamic> json) {
    return DiskUsage(
      rawOutputSize: json['raw_output_size'] as int? ?? 0,
      unpackedOutputSize: json['unpacked_output_size'] as int? ?? 0,
      convertedOutputSize: json['converted_output_size'] as int? ?? 0,
      availableSpace: json['available_space'] as int?,
    );
  }

  int get totalOutputSize => rawOutputSize + unpackedOutputSize + convertedOutputSize;

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
