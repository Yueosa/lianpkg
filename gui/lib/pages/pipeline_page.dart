/// Pipeline 流水线页 — 流程图 + 选项配置 + 启动 + 实时进度 + 结果
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

  const _PipelineUiState({
    this.runState = _RunState.idle,
    this.progress = const ProgressSnapshot(),
    this.output,
    this.errorMessage,
  });

  _PipelineUiState copyWith({
    _RunState? runState,
    ProgressSnapshot? progress,
    AutoOutput? output,
    String? errorMessage,
  }) {
    return _PipelineUiState(
      runState: runState ?? this.runState,
      progress: progress ?? this.progress,
      output: output ?? this.output,
      errorMessage: errorMessage ?? this.errorMessage,
    );
  }
}

class _PipelineNotifier extends StateNotifier<_PipelineUiState> {
  final Ref ref;
  Timer? _pollTimer;

  _PipelineNotifier(this.ref) : super(const _PipelineUiState());

  Future<void> start() async {
    if (state.runState == _RunState.running) return;

    // 从全局配置读取选项
    final config = await ref.read(configProvider.future);

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
        noRaw: !config.enableRawOutput,
        noTex: !config.pipeline.autoConvertTex,
        noCleanUnpacked: !config.cleanUnpacked,
        noIncremental: !config.pipeline.incremental,
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
      ref.invalidate(scanResultProvider);
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
    final config = ref.watch(configProvider).valueOrNull;
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.all(24),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('自动流水线', style: theme.textTheme.headlineMedium),
            const SizedBox(height: 8),
            Text(
              '一键完成壁纸处理：扫描 → 复制 / 解包 → 转换',
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 24),

            // ── 流程图 ──
            _FlowDiagram(
              enableRaw: config?.enableRawOutput ?? true,
              enableTex: config?.pipeline.autoConvertTex ?? true,
              enableClean: config?.cleanUnpacked ?? true,
              incremental: config?.pipeline.incremental ?? true,
              runState: uiState.runState,
              stage: uiState.progress.stage,
            ),
            const SizedBox(height: 24),

            // ── 选项 ──
            Text('运行选项', style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            _OptionTile(
              icon: Icons.file_copy_outlined,
              title: '复制 Raw',
              subtitle: '将非PKG壁纸的原始文件复制到 Raw 输出目录',
              value: config?.enableRawOutput ?? true,
              enabled: uiState.runState != _RunState.running,
              onChanged: (v) async {
                await ref
                    .read(lianpkgServiceProvider)
                    .setConfig('wallpaper.enable_raw_output', v.toString());
                ref.invalidate(configProvider);
              },
            ),
            _OptionTile(
              icon: Icons.image_outlined,
              title: '转换 TEX',
              subtitle: '将 TEX 纹理文件转换为标准图片/视频格式',
              value: config?.pipeline.autoConvertTex ?? true,
              enabled: uiState.runState != _RunState.running,
              onChanged: (v) async {
                await ref
                    .read(lianpkgServiceProvider)
                    .setConfig('pipeline.auto_convert_tex', v.toString());
                ref.invalidate(configProvider);
              },
            ),
            _OptionTile(
              icon: Icons.cleaning_services_outlined,
              title: '清理解包产物',
              subtitle: '转换完成后自动删除中间解包文件以节省空间',
              value: config?.cleanUnpacked ?? true,
              enabled: uiState.runState != _RunState.running,
              onChanged: (v) async {
                await ref
                    .read(lianpkgServiceProvider)
                    .setConfig('unpack.clean_unpacked', v.toString());
                ref.invalidate(configProvider);
              },
            ),
            _OptionTile(
              icon: Icons.fast_forward_outlined,
              title: '增量模式',
              subtitle: '跳过已处理过的壁纸，仅处理新增内容',
              value: config?.pipeline.incremental ?? true,
              enabled: uiState.runState != _RunState.running,
              onChanged: (v) async {
                await ref
                    .read(lianpkgServiceProvider)
                    .setConfig('pipeline.incremental', v.toString());
                ref.invalidate(configProvider);
              },
            ),
            const SizedBox(height: 20),

            // ── 启动按钮 ──
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

            // ── 进度 ──
            if (uiState.runState == _RunState.running) ...[
              _ProgressPanel(progress: uiState.progress),
            ],

            // ── 结果 ──
            if (uiState.runState == _RunState.done &&
                uiState.output != null) ...[
              _ResultPanel(output: uiState.output!),
            ],

            // ── 错误 ──
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
      ),
    );
  }
}

// =============================================================================
// 流程图
// =============================================================================

class _FlowDiagram extends StatelessWidget {
  final bool enableRaw;
  final bool enableTex;
  final bool enableClean;
  final bool incremental;
  final _RunState runState;
  final String stage;

  const _FlowDiagram({
    required this.enableRaw,
    required this.enableTex,
    required this.enableClean,
    required this.incremental,
    required this.runState,
    required this.stage,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isRunning = runState == _RunState.running;
    final isDone = runState == _RunState.done;

    final steps = <_StepData>[
      _StepData(
        icon: Icons.search,
        label: '扫描壁纸',
        enabled: true,
        active: isRunning && (stage.contains('scan') || stage.contains('Scan')),
        badge: incremental ? '增量' : null,
      ),
      _StepData(
        icon: Icons.file_copy_outlined,
        label: '复制 Raw',
        enabled: enableRaw,
        active:
            isRunning &&
            (stage.contains('copy') ||
                stage.contains('Copy') ||
                stage.contains('raw') ||
                stage.contains('Raw')),
      ),
      _StepData(
        icon: Icons.inventory_2_outlined,
        label: '解包 PKG',
        enabled: true,
        active:
            isRunning &&
            (stage.contains('unpack') ||
                stage.contains('Unpack') ||
                stage.contains('pkg') ||
                stage.contains('Pkg')),
      ),
      _StepData(
        icon: Icons.image_outlined,
        label: '转换 TEX',
        enabled: enableTex,
        active:
            isRunning &&
            (stage.contains('convert') ||
                stage.contains('Convert') ||
                stage.contains('tex') ||
                stage.contains('Tex')),
        badge: enableClean ? '清理' : null,
      ),
      _StepData(
        icon: Icons.check_circle_outlined,
        label: '完成',
        enabled: true,
        done: isDone,
      ),
    ];

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.account_tree_outlined,
                  size: 18,
                  color: theme.colorScheme.primary,
                ),
                const SizedBox(width: 8),
                Text('处理流程', style: theme.textTheme.titleMedium),
              ],
            ),
            const SizedBox(height: 16),
            SingleChildScrollView(
              scrollDirection: Axis.horizontal,
              child: Row(
                children: [
                  for (var i = 0; i < steps.length; i++) ...[
                    _StepBox(step: steps[i]),
                    if (i < steps.length - 1)
                      _Arrow(enabled: steps[i].enabled && steps[i + 1].enabled),
                  ],
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _StepData {
  final IconData icon;
  final String label;
  final bool enabled;
  final bool active;
  final bool done;
  final String? badge;

  const _StepData({
    required this.icon,
    required this.label,
    required this.enabled,
    this.active = false,
    this.done = false,
    this.badge,
  });
}

class _StepBox extends StatelessWidget {
  final _StepData step;
  const _StepBox({required this.step});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final color = step.done
        ? Colors.green
        : step.active
        ? theme.colorScheme.primary
        : step.enabled
        ? theme.colorScheme.onSurface
        : theme.colorScheme.outline.withAlpha(100);
    final bgColor = step.done
        ? Colors.green.withAlpha(30)
        : step.active
        ? theme.colorScheme.primaryContainer
        : step.enabled
        ? theme.colorScheme.surfaceContainerHighest
        : theme.colorScheme.surfaceContainerHighest.withAlpha(60);

    return Stack(
      clipBehavior: Clip.none,
      children: [
        AnimatedContainer(
          duration: const Duration(milliseconds: 300),
          width: 88,
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 12),
          decoration: BoxDecoration(
            color: bgColor,
            borderRadius: BorderRadius.circular(12),
            border: step.active || step.done
                ? Border.all(
                    color: step.done ? Colors.green : theme.colorScheme.primary,
                    width: 2,
                  )
                : null,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              if (step.active)
                SizedBox(
                  width: 28,
                  height: 28,
                  child: CircularProgressIndicator(
                    strokeWidth: 2.5,
                    color: theme.colorScheme.primary,
                  ),
                )
              else if (step.done)
                const Icon(Icons.check_circle, size: 28, color: Colors.green)
              else
                Icon(step.icon, size: 28, color: color),
              const SizedBox(height: 6),
              Text(
                step.label,
                textAlign: TextAlign.center,
                style: theme.textTheme.labelSmall?.copyWith(
                  color: color,
                  fontWeight: step.active ? FontWeight.bold : FontWeight.normal,
                ),
              ),
            ],
          ),
        ),
        // 角标
        if (step.badge != null)
          Positioned(
            top: -6,
            right: -6,
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: theme.colorScheme.tertiary,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                step.badge!,
                style: theme.textTheme.labelSmall?.copyWith(
                  color: theme.colorScheme.onTertiary,
                  fontSize: 10,
                ),
              ),
            ),
          ),
      ],
    );
  }
}

class _Arrow extends StatelessWidget {
  final bool enabled;
  const _Arrow({required this.enabled});

  @override
  Widget build(BuildContext context) {
    final color = enabled
        ? Theme.of(context).colorScheme.outline
        : Theme.of(context).colorScheme.outline.withAlpha(60);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 4),
      child: Icon(Icons.arrow_forward, size: 18, color: color),
    );
  }
}

// =============================================================================
// 选项 Tile
// =============================================================================

class _OptionTile extends StatelessWidget {
  final IconData icon;
  final String title;
  final String subtitle;
  final bool value;
  final bool enabled;
  final ValueChanged<bool> onChanged;

  const _OptionTile({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.value,
    required this.enabled,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 0,
      color: Theme.of(context).colorScheme.surfaceContainerLow,
      margin: const EdgeInsets.only(bottom: 4),
      child: SwitchListTile(
        secondary: Icon(icon),
        title: Text(title),
        subtitle: Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
        value: value,
        onChanged: enabled ? onChanged : null,
        dense: true,
      ),
    );
  }
}

// =============================================================================
// 进度面板
// =============================================================================

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

// =============================================================================
// 结果面板
// =============================================================================

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
                Text('处理完成', style: theme.textTheme.titleMedium),
                const Spacer(),
                Chip(
                  avatar: const Icon(Icons.timer_outlined, size: 16),
                  label: Text('${output.durationSecs.toStringAsFixed(1)}s'),
                ),
              ],
            ),
            const Divider(height: 24),

            // 统计行
            Wrap(
              spacing: 24,
              runSpacing: 12,
              children: [
                _ResultStat(
                  '壁纸',
                  output.stats.wallpapersProcessed,
                  icon: Icons.wallpaper,
                ),
                _ResultStat(
                  '跳过',
                  output.stats.wallpapersSkipped,
                  icon: Icons.skip_next,
                ),
                _ResultStat(
                  'PKG 解包',
                  output.stats.pkgsUnpacked,
                  icon: Icons.inventory_2,
                ),
                _ResultStat(
                  'TEX 转换',
                  output.stats.texsConverted,
                  icon: Icons.image,
                ),
              ],
            ),

            // 详细统计
            if (output.copyOutput != null ||
                output.pkgOutput != null ||
                output.texOutput != null) ...[
              const SizedBox(height: 16),
              const Divider(height: 1),
              const SizedBox(height: 12),
              Text('详细统计', style: theme.textTheme.labelMedium),
              const SizedBox(height: 8),
              Wrap(
                spacing: 24,
                runSpacing: 8,
                children: [
                  if (output.copyOutput != null) ...[
                    _ResultStat('Raw 复制', output.copyOutput!.copiedCount),
                    if (output.copyOutput!.skipped > 0)
                      _ResultStat('跳过', output.copyOutput!.skipped),
                  ],
                  if (output.pkgOutput != null) ...[
                    _ResultStat('解包文件', output.pkgOutput!.totalFiles),
                    _ResultStat('TEX 文件', output.pkgOutput!.texFiles),
                    if (output.pkgOutput!.pkgFailed > 0)
                      _ResultStat(
                        '解包错误',
                        output.pkgOutput!.pkgFailed,
                        isError: true,
                      ),
                  ],
                  if (output.texOutput != null) ...[
                    _ResultStat('图片', output.texOutput!.imageCount),
                    _ResultStat('视频', output.texOutput!.videoCount),
                    if (output.texOutput!.texFailed > 0)
                      _ResultStat(
                        'TEX 错误',
                        output.texOutput!.texFailed,
                        isError: true,
                      ),
                  ],
                ],
              ),
            ],
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
  final IconData? icon;

  const _ResultStat(this.label, this.value, {this.isError = false, this.icon});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final color = isError ? theme.colorScheme.error : null;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (icon != null) ...[
          Icon(icon, size: 16, color: color ?? theme.colorScheme.primary),
          const SizedBox(width: 4),
        ],
        Text(
          '$value',
          style: theme.textTheme.titleMedium?.copyWith(
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
        const SizedBox(width: 4),
        Text(label, style: theme.textTheme.labelSmall?.copyWith(color: color)),
      ],
    );
  }
}
