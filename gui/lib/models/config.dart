/// lianpkg 运行时配置模型
library;

class LianpkgConfig {
  final String workshopPath;
  final String rawOutputPath;
  final bool enableRawOutput;
  final String unpackedOutputPath;
  final bool cleanUnpacked;
  final String convertedOutputPath;
  final PipelineConfig pipeline;

  const LianpkgConfig({
    required this.workshopPath,
    required this.rawOutputPath,
    required this.enableRawOutput,
    required this.unpackedOutputPath,
    required this.cleanUnpacked,
    required this.convertedOutputPath,
    required this.pipeline,
  });

  factory LianpkgConfig.fromJson(Map<String, dynamic> json) {
    return LianpkgConfig(
      workshopPath: json['workshop_path'] as String? ?? '',
      rawOutputPath: json['raw_output_path'] as String? ?? '',
      enableRawOutput: json['enable_raw_output'] as bool? ?? true,
      unpackedOutputPath: json['unpacked_output_path'] as String? ?? '',
      cleanUnpacked: json['clean_unpacked'] as bool? ?? true,
      convertedOutputPath: json['converted_output_path'] as String? ?? '',
      pipeline: PipelineConfig.fromJson(
        json['pipeline'] as Map<String, dynamic>? ?? {},
      ),
    );
  }
}

class PipelineConfig {
  final bool incremental;
  final bool autoUnpackPkg;
  final bool autoConvertTex;

  const PipelineConfig({
    required this.incremental,
    required this.autoUnpackPkg,
    required this.autoConvertTex,
  });

  factory PipelineConfig.fromJson(Map<String, dynamic> json) {
    return PipelineConfig(
      incremental: json['incremental'] as bool? ?? true,
      autoUnpackPkg: json['auto_unpack_pkg'] as bool? ?? true,
      autoConvertTex: json['auto_convert_tex'] as bool? ?? true,
    );
  }
}
