/// State 状态管理页 — 已处理壁纸列表 + 按类型分组 + 清除状态
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/state.dart';
import '../providers/providers.dart';

/// 状态页类型筛选
final _stateFilterProvider = StateProvider<String?>((ref) => null);

class StatePage extends ConsumerWidget {
  const StatePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final stateAsync = ref.watch(stateProvider);
    final filter = ref.watch(_stateFilterProvider);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 标题栏
          Row(
            children: [
              Text('处理状态', style: theme.textTheme.headlineMedium),
              const Spacer(),
              OutlinedButton.icon(
                onPressed: () => ref.invalidate(stateProvider),
                icon: const Icon(Icons.refresh),
                label: const Text('刷新'),
              ),
              const SizedBox(width: 8),
              FilledButton.tonalIcon(
                onPressed: () => _confirmClear(context, ref),
                icon: const Icon(Icons.delete_sweep),
                label: const Text('清除全部'),
              ),
            ],
          ),
          const SizedBox(height: 16),

          // 筛选 chips
          Wrap(
            spacing: 8,
            children: [
              for (final type in ['Pkg', 'PkgTex', 'Raw', 'Skipped'])
                FilterChip(
                  label: Text(type),
                  selected: filter == type,
                  onSelected: (sel) {
                    ref.read(_stateFilterProvider.notifier).state = sel
                        ? type
                        : null;
                  },
                ),
            ],
          ),
          const SizedBox(height: 16),

          // 列表
          Expanded(
            child: stateAsync.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Center(child: Text('加载失败: $e')),
              data: (stateData) {
                var entries = stateData.processed.entries.toList();

                if (filter != null) {
                  entries = entries
                      .where((e) => e.value.processType == filter)
                      .toList();
                }

                // 按处理时间倒序
                entries.sort(
                  (a, b) => b.value.processedAt.compareTo(a.value.processedAt),
                );

                if (entries.isEmpty) {
                  return const Center(child: Text('没有处理记录'));
                }

                return ListView.builder(
                  itemCount: entries.length,
                  itemBuilder: (ctx, i) {
                    final id = entries[i].key;
                    final entry = entries[i].value;
                    return _ProcessedTile(id: id, entry: entry);
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmClear(BuildContext context, WidgetRef ref) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('清除所有状态'),
        content: const Text('这将清空 state.json 中所有已处理记录。此操作不可撤销。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('清除'),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    try {
      await ref.read(lianpkgServiceProvider).clearState();
      ref.invalidate(stateProvider);
      ref.invalidate(statusProvider);
      if (context.mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('状态已清除')));
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('清除失败: $e'), backgroundColor: Colors.red),
        );
      }
    }
  }
}

class _ProcessedTile extends StatelessWidget {
  final String id;
  final ProcessedEntry entry;

  const _ProcessedTile({required this.id, required this.entry});

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
        child: Icon(icon, color: color, size: 20),
      ),
      title: Text(
        entry.title ?? id,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
      ),
      subtitle: Text('$id · ${entry.processType}'),
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
