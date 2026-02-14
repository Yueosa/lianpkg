/// Pipeline 流水线页 — 选项配置 + 启动 + 实时进度 + 结果
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/pipeline.dart';
import '../providers/providers.dart';

/// 流水线运行状态
final _pipelineStateProvider =
    StateNotifierProvider<_PipelineNotifier, _PipelineUiState>((ref) {
      return _PipelineNotifier(ref);
    });

enum _RunState { idle, running, done, error }

class _PipelineUiState {
  final _RunState runState;
  final ProgressSnapshot progress;
  final AutoOutput? output;
  final String? errorMessage;

  // 选项
  final bool noRaw;
  final bool noTex;
  final bool noCleanUnpacked;
  final bool noIncremental;

  const _PipelineUiState({
    this.runState = _RunState.idle,
    this.progress = const ProgressSnapshot(),
    this.output,
    this.errorMessage,
    this.noRaw = false,
    this.noTex = false,
    this.noCleanUnpacked = false,
    this.noIncremental = false,
  });

  _PipelineUiState copyWith({
    _RunState? runState,
    ProgressSnapshot? progress,
    AutoOutput? output,
    String? errorMessage,
    bool? noRaw,
    bool? noTex,
    bool? noCleanUnpacked,
    bool? noIncremental,
  }) {
    return _PipelineUiState(
      runState: runState ?? this.runState,
      progress: progress ?? this.progress,
      output: output ?? this.output,
      errorMessage: errorMessage ?? this.errorMessage,
      noRaw: noRaw ?? this.noRaw,
      noTex: noTex ?? this.noTex,
      noCleanUnpacked: noCleanUnpacked ?? this.noCleanUnpacked,
      noIncremental: noIncremental ?? this.noIncremental,
    );
  }
}

class _PipelineNotifier extends StateNotifier<_PipelineUiState> {
  final Ref ref;
  Timer? _pollTimer;

  _PipelineNotifier(this.ref) : super(const _PipelineUiState());

  void toggleNoRaw(bool v) => state = state.copyWith(noRaw: v);
  void toggleNoTex(bool v) => state = state.copyWith(noTex: v);
  void toggleNoCleanUnpacked(bool v) =>
      state = state.copyWith(noCleanUnpacked: v);
  void toggleNoIncremental(bool v) => state = state.copyWith(noIncremental: v);

  Future<void> start() async {
    if (state.runState == _RunState.running) return;

    state = state.copyWith(
      runState: _RunState.running,
      progress: const ProgressSnapshot(running: true),
      output: null,
      errorMessage: null,
    );

    // 启动进度轮询
    _pollTimer = Timer.periodic(const Duration(milliseconds: 200), (_) {
      _pollProgress();
    });

    try {
      final service = ref.read(lianpkgServiceProvider);
      final output = await service.runAuto(
        noRaw: state.noRaw,
        noTex: state.noTex,
        noCleanUnpacked: state.noCleanUnpacked,
        noIncremental: state.noIncremental,
      );

      _pollTimer?.cancel();
      state = state.copyWith(
        runState: _RunState.done,
        output: output,
        progress: const ProgressSnapshot(percent: 100),
      );

      // 刷新关联数据
      ref.invalidate(statusProvider);
      ref.invalidate(stateProvider);
    } catch (e) {
      _pollTimer?.cancel();
      state = state.copyWith(
        runState: _RunState.error,
        errorMessage: e.toString(),
      );
    }
  }

  void _pollProgress() {
    try {
      final service = ref.read(lianpkgServiceProvider);
      final snap = service.pollProgress();
      state = state.copyWith(progress: snap);
    } catch (_) {
      // 轮询失败不中断流水线
    }
  }

  void reset() {
    _pollTimer?.cancel();
    state = const _PipelineUiState();
  }

  @override
  void dispose() {
    _pollTimer?.cancel();
    super.dispose();
  }
}

class PipelinePage extends ConsumerWidget {
  const PipelinePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final uiState = ref.watch(_pipelineStateProvider);
    final notifier = ref.read(_pipelineStateProvider.notifier);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('自动流水线', style: theme.textTheme.headlineMedium),
          const SizedBox(height: 20),

          // 选项
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('选项', style: theme.textTheme.titleMedium),
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 16,
                    children: [
                      _OptionSwitch(
                        label: '复制 Raw',
                        value: !uiState.noRaw,
                        onChanged: uiState.runState != _RunState.running
                            ? (v) => notifier.toggleNoRaw(!v)
                            : null,
                      ),
                      _OptionSwitch(
                        label: '转换 TEX',
                        value: !uiState.noTex,
                        onChanged: uiState.runState != _RunState.running
                            ? (v) => notifier.toggleNoTex(!v)
                            : null,
                      ),
                      _OptionSwitch(
                        label: '清理解包',
                        value: !uiState.noCleanUnpacked,
                        onChanged: uiState.runState != _RunState.running
                            ? (v) => notifier.toggleNoCleanUnpacked(!v)
                            : null,
                      ),
                      _OptionSwitch(
                        label: '增量模式',
                        value: !uiState.noIncremental,
                        onChanged: uiState.runState != _RunState.running
                            ? (v) => notifier.toggleNoIncremental(!v)
                            : null,
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),

          // 启动按钮
          Row(
            children: [
              if (uiState.runState != _RunState.running)
                FilledButton.icon(
                  onPressed: () => notifier.start(),
                  icon: const Icon(Icons.play_arrow),
                  label: const Text('开始处理'),
                )
              else
                FilledButton.icon(
                  onPressed: null,
                  icon: const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
                  label: const Text('处理中...'),
                ),
              if (uiState.runState == _RunState.done ||
                  uiState.runState == _RunState.error) ...[
                const SizedBox(width: 12),
                OutlinedButton.icon(
                  onPressed: () => notifier.reset(),
                  icon: const Icon(Icons.restart_alt),
                  label: const Text('重置'),
                ),
              ],
            ],
          ),
          const SizedBox(height: 24),

          // 进度
          if (uiState.runState == _RunState.running) ...[
            _ProgressPanel(progress: uiState.progress),
          ],

          // 结果
          if (uiState.runState == _RunState.done && uiState.output != null) ...[
            _ResultPanel(output: uiState.output!),
          ],

          // 错误
          if (uiState.runState == _RunState.error) ...[
            Card(
              color: theme.colorScheme.errorContainer,
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Row(
                  children: [
                    Icon(Icons.error, color: theme.colorScheme.error),
                    const SizedBox(width: 12),
                    Expanded(child: Text(uiState.errorMessage ?? '未知错误')),
                  ],
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _OptionSwitch extends StatelessWidget {
  final String label;
  final bool value;
  final ValueChanged<bool>? onChanged;

  const _OptionSwitch({
    required this.label,
    required this.value,
    this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Switch(value: value, onChanged: onChanged),
        const SizedBox(width: 4),
        Text(label),
      ],
    );
  }
}

class _ProgressPanel extends StatelessWidget {
  final ProgressSnapshot progress;
  const _ProgressPanel({required this.progress});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text('阶段: ', style: theme.textTheme.titleSmall),
                Text(progress.stage.isNotEmpty ? progress.stage : '...'),
                const Spacer(),
                Text('${progress.percent}%', style: theme.textTheme.titleSmall),
              ],
            ),
            const SizedBox(height: 8),
            LinearProgressIndicator(value: progress.percent / 100),
            if (progress.currentItem != null) ...[
              const SizedBox(height: 8),
              Text(
                progress.currentItem!,
                style: theme.textTheme.bodySmall,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ],
            if (progress.message.isNotEmpty) ...[
              const SizedBox(height: 4),
              Text(progress.message, style: theme.textTheme.bodySmall),
            ],
          ],
        ),
      ),
    );
  }
}

class _ResultPanel extends StatelessWidget {
  final AutoOutput output;
  const _ResultPanel({required this.output});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      color: theme.colorScheme.primaryContainer.withAlpha(80),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.check_circle, color: theme.colorScheme.primary),
                const SizedBox(width: 8),
                Text('完成', style: theme.textTheme.titleMedium),
                const Spacer(),
                Text(
                  '${output.durationSecs.toStringAsFixed(1)}s',
                  style: theme.textTheme.bodyMedium,
                ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 24,
              runSpacing: 8,
              children: [
                _ResultStat('壁纸', output.stats.wallpapersProcessed),
                _ResultStat('跳过', output.stats.wallpapersSkipped),
                _ResultStat('PKG 解包', output.stats.pkgsUnpacked),
                _ResultStat('TEX 转换', output.stats.texsConverted),
                if (output.copyOutput != null) ...[
                  _ResultStat('Raw 复制', output.copyOutput!.copiedCount),
                  if (output.copyOutput!.errorCount > 0)
                    _ResultStat(
                      'Raw 错误',
                      output.copyOutput!.errorCount,
                      isError: true,
                    ),
                ],
                if (output.pkgOutput != null) ...[
                  _ResultStat('解包数', output.pkgOutput!.unpackedCount),
                  if (output.pkgOutput!.errorCount > 0)
                    _ResultStat(
                      '解包错误',
                      output.pkgOutput!.errorCount,
                      isError: true,
                    ),
                ],
                if (output.texOutput != null) ...[
                  _ResultStat('图片', output.texOutput!.imageCount),
                  _ResultStat('视频', output.texOutput!.videoCount),
                  if (output.texOutput!.errorCount > 0)
                    _ResultStat(
                      'TEX 错误',
                      output.texOutput!.errorCount,
                      isError: true,
                    ),
                ],
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _ResultStat extends StatelessWidget {
  final String label;
  final int value;
  final bool isError;

  const _ResultStat(this.label, this.value, {this.isError = false});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          '$value',
          style: Theme.of(context).textTheme.titleLarge?.copyWith(
            fontWeight: FontWeight.bold,
            color: isError ? Theme.of(context).colorScheme.error : null,
          ),
        ),
        Text(label, style: Theme.of(context).textTheme.labelSmall),
      ],
    );
  }
}
