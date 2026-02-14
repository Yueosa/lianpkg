/// Browser 壁纸浏览页 — 网格布局 + 搜索筛选 + 缩略图
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/wallpaper.dart';
import '../providers/providers.dart';

/// 搜索关键词
final _searchQueryProvider = StateProvider<String>((ref) => '');

/// 类型筛选
final _filterTypeProvider = StateProvider<WallpaperType?>((ref) => null);

/// 过滤后的壁纸列表
final _filteredWallpapersProvider = Provider<AsyncValue<List<WallpaperInfo>>>((
  ref,
) {
  final scanAsync = ref.watch(scanResultProvider);
  final query = ref.watch(_searchQueryProvider).toLowerCase();
  final filterType = ref.watch(_filterTypeProvider);

  return scanAsync.whenData((scan) {
    var list = scan.wallpapers;

    if (filterType != null) {
      list = list.where((w) => w.wallpaperType == filterType).toList();
    }

    if (query.isNotEmpty) {
      list = list.where((w) {
        final title = (w.title ?? '').toLowerCase();
        final id = w.id.toLowerCase();
        return title.contains(query) || id.contains(query);
      }).toList();
    }

    return list;
  });
});

class BrowserPage extends ConsumerWidget {
  const BrowserPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final wallpapersAsync = ref.watch(_filteredWallpapersProvider);
    final theme = Theme.of(context);
    final filterType = ref.watch(_filterTypeProvider);

    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // 搜索栏 + 筛选 chips
          Row(
            children: [
              Expanded(
                child: SearchBar(
                  hintText: '搜索壁纸名称或 ID...',
                  leading: const Icon(Icons.search),
                  onChanged: (v) =>
                      ref.read(_searchQueryProvider.notifier).state = v,
                ),
              ),
              const SizedBox(width: 12),
              ...WallpaperType.values.map(
                (type) => Padding(
                  padding: const EdgeInsets.only(left: 4),
                  child: FilterChip(
                    label: Text(type.label),
                    selected: filterType == type,
                    onSelected: (selected) {
                      ref.read(_filterTypeProvider.notifier).state = selected
                          ? type
                          : null;
                    },
                  ),
                ),
              ),
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.refresh),
                onPressed: () => ref.invalidate(scanResultProvider),
                tooltip: '重新扫描',
              ),
            ],
          ),
          const SizedBox(height: 16),

          // 壁纸网格
          Expanded(
            child: wallpapersAsync.when(
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (e, _) => Center(
                child: Text(
                  '扫描失败: $e',
                  style: TextStyle(color: theme.colorScheme.error),
                ),
              ),
              data: (wallpapers) {
                if (wallpapers.isEmpty) {
                  return const Center(child: Text('没有找到壁纸'));
                }
                return GridView.builder(
                  gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                    maxCrossAxisExtent: 240,
                    childAspectRatio: 0.85,
                    crossAxisSpacing: 12,
                    mainAxisSpacing: 12,
                  ),
                  itemCount: wallpapers.length,
                  itemBuilder: (ctx, i) =>
                      _WallpaperCard(wallpaper: wallpapers[i]),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

class _WallpaperCard extends StatelessWidget {
  final WallpaperInfo wallpaper;

  const _WallpaperCard({required this.wallpaper});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: () {
          // TODO: Phase C — 跳转到 Detail 页
        },
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // 缩略图
            Expanded(child: _buildPreview()),
            // 信息
            Padding(
              padding: const EdgeInsets.all(8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    wallpaper.title ?? wallpaper.id,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Row(
                    children: [
                      _TypeBadge(type: wallpaper.wallpaperType),
                      const SizedBox(width: 4),
                      if (wallpaper.processed)
                        Icon(
                          Icons.check_circle,
                          size: 14,
                          color: theme.colorScheme.primary,
                        ),
                      const Spacer(),
                      Text(
                        wallpaper.id,
                        style: theme.textTheme.labelSmall?.copyWith(
                          color: theme.colorScheme.outline,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPreview() {
    if (wallpaper.previewPath != null) {
      final file = File(wallpaper.previewPath!);
      if (file.existsSync()) {
        return Image.file(file, fit: BoxFit.cover);
      }
    }
    return Container(
      color: Colors.grey.shade200,
      child: const Icon(
        Icons.image_not_supported,
        size: 48,
        color: Colors.grey,
      ),
    );
  }
}

class _TypeBadge extends StatelessWidget {
  final WallpaperType type;
  const _TypeBadge({required this.type});

  @override
  Widget build(BuildContext context) {
    final (color, bg) = switch (type) {
      WallpaperType.pkg => (Colors.blue.shade700, Colors.blue.shade50),
      WallpaperType.raw => (Colors.green.shade700, Colors.green.shade50),
      WallpaperType.skipped => (Colors.grey.shade600, Colors.grey.shade100),
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        color: bg,
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        type.label,
        style: TextStyle(
          fontSize: 10,
          fontWeight: FontWeight.bold,
          color: color,
        ),
      ),
    );
  }
}
