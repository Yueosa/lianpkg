/// Dashboard 首页 — 统计卡片 + 快捷操作 + 最近记录
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/state.dart';
import '../providers/providers.dart';

class DashboardPage extends ConsumerWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statusAsync = ref.watch(statusProvider);
    final stateAsync = ref.watch(stateProvider);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.all(24),
      child: statusAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.error_outline,
                size: 48,
                color: theme.colorScheme.error,
              ),
              const SizedBox(height: 12),
              Text('加载失败: $e', style: theme.textTheme.bodyLarge),
              const SizedBox(height: 12),
              FilledButton.icon(
                onPressed: () => ref.invalidate(statusProvider),
                icon: const Icon(Icons.refresh),
                label: const Text('重试'),
              ),
            ],
          ),
        ),
        data: (status) => SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('总览', style: theme.textTheme.headlineMedium),
              const SizedBox(height: 16),

              // 统计卡片
              Row(
                children: [
                  _StatCard(
                    icon: Icons.wallpaper,
                    label: '已处理',
                    value: '${status.totalProcessed}',
                    color: theme.colorScheme.primary,
                  ),
                  const SizedBox(width: 12),
                  _StatCard(
                    icon: Icons.inventory_2,
                    label: 'PKG',
                    value: '${status.pkgCount}',
                    color: theme.colorScheme.tertiary,
                  ),
                  const SizedBox(width: 12),
                  _StatCard(
                    icon: Icons.file_copy,
                    label: 'Raw',
                    value: '${status.rawCount}',
                    color: theme.colorScheme.secondary,
                  ),
                  const SizedBox(width: 12),
                  _StatCard(
                    icon: Icons.skip_next,
                    label: '已跳过',
                    value: '${status.skippedCount}',
                    color: theme.colorScheme.outline,
                  ),
                ],
              ),
              const SizedBox(height: 24),

              // 磁盘用量估算
              _DiskEstimateCard(estimate: status.diskEstimate),
              const SizedBox(height: 24),

              // 快捷操作
              Text('快捷操作', style: theme.textTheme.titleLarge),
              const SizedBox(height: 12),
              Wrap(
                spacing: 12,
                children: [
                  FilledButton.icon(
                    onPressed: () {
                      ref.read(navigationIndexProvider.notifier).state = 2;
                    },
                    icon: const Icon(Icons.play_arrow),
                    label: const Text('一键处理'),
                  ),
                  OutlinedButton.icon(
                    onPressed: () {
                      ref.read(navigationIndexProvider.notifier).state = 1;
                    },
                    icon: const Icon(Icons.photo_library),
                    label: const Text('浏览壁纸'),
                  ),
                  OutlinedButton.icon(
                    onPressed: () {
                      ref.invalidate(statusProvider);
                      ref.invalidate(stateProvider);
                    },
                    icon: const Icon(Icons.refresh),
                    label: const Text('刷新'),
                  ),
                ],
              ),
              const SizedBox(height: 24),

              // 最近处理记录
              Text('最近记录', style: theme.textTheme.titleLarge),
              const SizedBox(height: 8),
              stateAsync.when(
                loading: () => const Padding(
                  padding: EdgeInsets.all(16),
                  child: Center(child: CircularProgressIndicator()),
                ),
                error: (e, _) => Text('加载失败: $e'),
                data: (stateData) {
                  final entries = stateData.processed.entries.toList()
                    ..sort(
                      (a, b) =>
                          b.value.processedAt.compareTo(a.value.processedAt),
                    );
                  final recent = entries.take(5).toList();

                  if (recent.isEmpty) {
                    return const Padding(
                      padding: EdgeInsets.all(16),
                      child: Text('暂无处理记录'),
                    );
                  }

                  return Card(
                    child: Column(
                      children: recent.map((e) {
                        return _RecentTile(id: e.key, entry: e.value);
                      }).toList(),
                    ),
                  );
                },
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _StatCard extends StatelessWidget {
  final IconData icon;
  final String label;
  final String value;
  final Color color;

  const _StatCard({
    required this.icon,
    required this.label,
    required this.value,
    required this.color,
  });

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            children: [
              Icon(icon, size: 28, color: color),
              const SizedBox(height: 8),
              Text(
                value,
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                  fontWeight: FontWeight.bold,
                  color: color,
                ),
              ),
              const SizedBox(height: 4),
              Text(label, style: Theme.of(context).textTheme.bodySmall),
            ],
          ),
        ),
      ),
    );
  }
}

class _DiskEstimateCard extends StatelessWidget {
  final DiskEstimate estimate;
  const _DiskEstimateCard({required this.estimate});

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
                const Icon(Icons.storage, size: 20),
                const SizedBox(width: 8),
                Text('磁盘估算', style: theme.textTheme.titleMedium),
                const Spacer(),
                if (estimate.spaceSufficient)
                  Chip(
                    label: const Text('空间充足'),
                    avatar: const Icon(Icons.check, size: 16),
                    backgroundColor: Colors.green.shade50,
                    labelStyle: TextStyle(color: Colors.green.shade700),
                  )
                else
                  Chip(
                    label: const Text('空间不足'),
                    avatar: const Icon(Icons.warning, size: 16),
                    backgroundColor: Colors.red.shade50,
                    labelStyle: TextStyle(color: Colors.red.shade700),
                  ),
              ],
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 24,
              runSpacing: 8,
              children: [
                _DiskStat('PKG 源', DiskEstimate.formatBytes(estimate.pkgSize)),
                _DiskStat('Raw 源', DiskEstimate.formatBytes(estimate.rawSize)),
                _DiskStat(
                  '预估解包',
                  DiskEstimate.formatBytes(estimate.estimatedUnpacked),
                ),
                _DiskStat(
                  '预估转换',
                  DiskEstimate.formatBytes(estimate.estimatedConverted),
                ),
                _DiskStat(
                  '峰值用量',
                  DiskEstimate.formatBytes(estimate.estimatedPeak),
                ),
                _DiskStat(
                  '最终用量',
                  DiskEstimate.formatBytes(estimate.estimatedFinal),
                ),
                if (estimate.availableSpace != null)
                  _DiskStat(
                    '可用空间',
                    DiskEstimate.formatBytes(estimate.availableSpace!),
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _DiskStat extends StatelessWidget {
  final String label;
  final String value;
  const _DiskStat(this.label, this.value);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          value,
          style: Theme.of(
            context,
          ).textTheme.titleSmall?.copyWith(fontWeight: FontWeight.bold),
        ),
        Text(label, style: Theme.of(context).textTheme.labelSmall),
      ],
    );
  }
}

class _RecentTile extends StatelessWidget {
  final String id;
  final ProcessedEntry entry;
  const _RecentTile({required this.id, required this.entry});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final (icon, color) = switch (entry.processType) {
      'Pkg' || 'PkgTex' => (Icons.inventory_2, Colors.blue),
      'Raw' => (Icons.file_copy, Colors.green),
      _ => (Icons.skip_next, Colors.grey),
    };

    return ListTile(
      leading: CircleAvatar(
        backgroundColor: color.withAlpha(30),
        child: Icon(icon, color: color, size: 18),
      ),
      title: Text(
        entry.title ?? id,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text(entry.processType),
      trailing: Text(
        _formatTime(entry.processedAt),
        style: theme.textTheme.labelSmall,
      ),
    );
  }

  String _formatTime(String iso) {
    try {
      final dt = DateTime.parse(iso).toLocal();
      return '${dt.month}/${dt.day} ${dt.hour.toString().padLeft(2, '0')}:'
          '${dt.minute.toString().padLeft(2, '0')}';
    } catch (_) {
      return iso;
    }
  }
}
