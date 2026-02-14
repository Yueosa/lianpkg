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
  final Map<String, dynamic> diskEstimate;

  const StatusInfo({
    this.totalProcessed = 0,
    this.pkgCount = 0,
    this.rawCount = 0,
    this.skippedCount = 0,
    this.lastRun,
    this.diskEstimate = const {},
  });

  factory StatusInfo.fromJson(Map<String, dynamic> json) {
    return StatusInfo(
      totalProcessed: json['total_processed'] as int? ?? 0,
      pkgCount: json['pkg_count'] as int? ?? 0,
      rawCount: json['raw_count'] as int? ?? 0,
      skippedCount: json['skipped_count'] as int? ?? 0,
      lastRun: json['last_run'] as String?,
      diskEstimate: json['disk_estimate'] as Map<String, dynamic>? ?? {},
    );
  }
}
