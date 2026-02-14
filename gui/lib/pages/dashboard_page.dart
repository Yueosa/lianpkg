/// Dashboard 首页 — 统计卡片 + 快捷操作 + 最近记录
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/providers.dart';

class DashboardPage extends ConsumerWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final statusAsync = ref.watch(statusProvider);
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
        data: (status) => Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Overview', style: theme.textTheme.headlineMedium),
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
                  label: 'Skipped',
                  value: '${status.skippedCount}',
                  color: theme.colorScheme.outline,
                ),
              ],
            ),
            const SizedBox(height: 32),

            // 快捷操作
            Text('Quick Actions', style: theme.textTheme.titleLarge),
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
                  onPressed: () => ref.invalidate(statusProvider),
                  icon: const Icon(Icons.refresh),
                  label: const Text('刷新'),
                ),
              ],
            ),
            const SizedBox(height: 32),

            // 最近运行
            if (status.lastRun != null) ...[
              Text('Last Run', style: theme.textTheme.titleLarge),
              const SizedBox(height: 8),
              Text(status.lastRun!, style: theme.textTheme.bodyMedium),
            ],
          ],
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
