/// Detail 壁纸详情页 — 预览 + 元数据 + PKG 文件树 + TEX 元数据 + 操作按钮
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/pkg_preview.dart';
import '../models/tex_preview.dart';
import '../models/wallpaper.dart';
import '../providers/providers.dart';
import '../services/lianpkg_service.dart' show PkgSourceDto;

class DetailPage extends ConsumerStatefulWidget {
  final WallpaperInfo wallpaper;

  const DetailPage({super.key, required this.wallpaper});

  @override
  ConsumerState<DetailPage> createState() => _DetailPageState();
}

class _DetailPageState extends ConsumerState<DetailPage> {
  WallpaperInfo get wallpaper => widget.wallpaper;

  // PKG 预览缓存：pkgPath → Future<PkgPreview>
  final Map<String, Future<PkgPreview>> _pkgPreviews = {};
  // TEX 预览缓存：texPath → Future<TexPreview>
  final Map<String, Future<TexPreview>> _texPreviews = {};

  // 操作状态
  bool _unpacking = false;
  bool _converting = false;
  String? _unpackResult;
  String? _convertResult;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: Text(wallpaper.title ?? wallpaper.id),
        actions: [
          IconButton(
            icon: const Icon(Icons.copy),
            tooltip: '复制 ID',
            onPressed: () {
              Clipboard.setData(ClipboardData(text: wallpaper.id));
              ScaffoldMessenger.of(context).showSnackBar(
                const SnackBar(
                  content: Text('已复制壁纸 ID'),
                  duration: Duration(seconds: 1),
                ),
              );
            },
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // ── 顶部：预览图 + 元信息 ──
            _buildHeader(theme),
            const SizedBox(height: 24),

            // ── 操作按钮 ──
            if (wallpaper.hasPkg) ...[
              _buildActions(theme),
              const SizedBox(height: 24),
            ],

            // ── PKG 文件列表 ──
            if (wallpaper.pkgFiles.isNotEmpty) ...[
              _buildPkgSection(theme),
              const SizedBox(height: 24),
            ],

            // ── 文件夹路径 ──
            _buildFolderInfo(theme),
          ],
        ),
      ),
    );
  }

  // ============================================================================
  // 顶部元信息
  // ============================================================================

  Widget _buildHeader(ThemeData theme) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // 预览图
        ClipRRect(
          borderRadius: BorderRadius.circular(12),
          child: SizedBox(width: 280, height: 158, child: _buildPreviewImage()),
        ),
        const SizedBox(width: 24),
        // 元信息
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                wallpaper.title ?? '(无标题)',
                style: theme.textTheme.headlineSmall?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
              const SizedBox(height: 8),
              _MetaRow(label: 'ID', value: wallpaper.id),
              if (wallpaper.wallpaperType != null)
                _MetaRow(label: '类型', value: wallpaper.wallpaperType!),
              _MetaRow(label: '分类', value: wallpaper.category.label),
              _MetaRow(label: '已处理', value: wallpaper.isProcessed ? '是' : '否'),
              _MetaRow(label: 'PKG 文件数', value: '${wallpaper.pkgFiles.length}'),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildPreviewImage() {
    if (wallpaper.previewPath != null) {
      final file = File(wallpaper.previewPath!);
      if (file.existsSync()) {
        return Image.file(file, fit: BoxFit.cover);
      }
    }
    return Container(
      color: Colors.grey.shade200,
      child: const Center(
        child: Icon(Icons.image_not_supported, size: 48, color: Colors.grey),
      ),
    );
  }

  // ============================================================================
  // 操作按钮
  // ============================================================================

  Widget _buildActions(ThemeData theme) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('操作', style: theme.textTheme.titleMedium),
            const SizedBox(height: 12),
            Wrap(
              spacing: 12,
              runSpacing: 8,
              children: [
                FilledButton.icon(
                  onPressed: _unpacking ? null : _doUnpack,
                  icon: _unpacking
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.unarchive),
                  label: Text(_unpacking ? '解包中...' : '解包 PKG'),
                ),
                FilledButton.tonalIcon(
                  onPressed: _converting ? null : _doConvert,
                  icon: _converting
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.transform),
                  label: Text(_converting ? '转换中...' : '转换 TEX'),
                ),
              ],
            ),
            if (_unpackResult != null) ...[
              const SizedBox(height: 8),
              Text(
                _unpackResult!,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.primary,
                ),
              ),
            ],
            if (_convertResult != null) ...[
              const SizedBox(height: 8),
              Text(
                _convertResult!,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.tertiary,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Future<void> _doUnpack() async {
    setState(() {
      _unpacking = true;
      _unpackResult = null;
    });
    try {
      final service = ref.read(lianpkgServiceProvider);
      final config = await ref.read(configProvider.future);
      final output = await service.unpackPkg(
        sources: [
          PkgSourceDto(wallpaperId: wallpaper.id, pkgPaths: wallpaper.pkgFiles),
        ],
        output: config.unpackedOutputPath,
      );
      if (mounted) {
        setState(() {
          _unpackResult =
              '解包完成: ${output.totalFiles} 个文件 '
              '(${output.pkgSuccess} 个 PKG 成功, '
              '${output.texFiles} 个 TEX)'
              '${output.pkgFailed > 0 ? ', ${output.pkgFailed} 个失败' : ''}';
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _unpackResult = '解包失败: $e');
      }
    } finally {
      if (mounted) setState(() => _unpacking = false);
    }
  }

  Future<void> _doConvert() async {
    setState(() {
      _converting = true;
      _convertResult = null;
    });
    try {
      final service = ref.read(lianpkgServiceProvider);
      final config = await ref.read(configProvider.future);
      final output = await service.convertTex(input: config.unpackedOutputPath);
      if (mounted) {
        setState(() {
          _convertResult =
              '转换完成: ${output.imageCount} 张图片, '
              '${output.videoCount} 个视频'
              '${output.texFailed > 0 ? ', ${output.texFailed} 个错误' : ''}';
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => _convertResult = '转换失败: $e');
      }
    } finally {
      if (mounted) setState(() => _converting = false);
    }
  }

  // ============================================================================
  // PKG 文件列表
  // ============================================================================

  Widget _buildPkgSection(ThemeData theme) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'PKG 文件 (${wallpaper.pkgFiles.length})',
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            ...wallpaper.pkgFiles.map(
              (pkgPath) => _PkgFileTile(
                pkgPath: pkgPath,
                previewFuture: _getOrFetchPkgPreview(pkgPath),
                onTexTap: (texPath) => _showTexPreview(context, texPath),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<PkgPreview> _getOrFetchPkgPreview(String path) {
    return _pkgPreviews.putIfAbsent(path, () {
      final service = ref.read(lianpkgServiceProvider);
      return service.previewPkg(path);
    });
  }

  void _showTexPreview(BuildContext context, String texPath) {
    final future = _texPreviews.putIfAbsent(texPath, () {
      final service = ref.read(lianpkgServiceProvider);
      return service.previewTex(texPath);
    });

    showDialog(
      context: context,
      builder: (ctx) => FutureBuilder<TexPreview>(
        future: future,
        builder: (ctx, snap) {
          if (snap.connectionState != ConnectionState.done) {
            return const AlertDialog(
              content: SizedBox(
                height: 80,
                child: Center(child: CircularProgressIndicator()),
              ),
            );
          }
          if (snap.hasError) {
            return AlertDialog(
              title: const Text('TEX 预览失败'),
              content: Text('${snap.error}'),
              actions: [
                TextButton(
                  onPressed: () => Navigator.pop(ctx),
                  child: const Text('关闭'),
                ),
              ],
            );
          }
          final tex = snap.data!;
          return AlertDialog(
            title: Text(texPath.split('/').last),
            content: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _TexInfoRow('版本', tex.version),
                _TexInfoRow('格式', tex.format),
                _TexInfoRow('分辨率', tex.resolution),
                _TexInfoRow('类型', tex.typeLabel),
                _TexInfoRow('图片数', '${tex.imageCount}'),
                _TexInfoRow('Mipmap', '${tex.mipmapCount}'),
                _TexInfoRow('压缩', tex.isCompressed ? '是' : '否'),
                _TexInfoRow('数据大小', tex.formattedDataSize),
                _TexInfoRow('推荐输出', tex.recommendedOutput),
              ],
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(ctx),
                child: const Text('关闭'),
              ),
            ],
          );
        },
      ),
    );
  }

  // ============================================================================
  // 文件夹信息
  // ============================================================================

  Widget _buildFolderInfo(ThemeData theme) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            const Icon(Icons.folder_open, size: 20),
            const SizedBox(width: 8),
            Expanded(
              child: SelectableText(
                wallpaper.folderPath,
                style: theme.textTheme.bodySmall?.copyWith(
                  fontFamily: 'monospace',
                ),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.copy, size: 16),
              iconSize: 16,
              tooltip: '复制路径',
              onPressed: () {
                Clipboard.setData(ClipboardData(text: wallpaper.folderPath));
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('已复制路径'),
                    duration: Duration(seconds: 1),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}

// =============================================================================
// 子组件
// =============================================================================

class _MetaRow extends StatelessWidget {
  final String label;
  final String value;
  const _MetaRow({required this.label, required this.value});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Theme.of(context).colorScheme.outline,
              ),
            ),
          ),
          Expanded(
            child: Text(value, style: Theme.of(context).textTheme.bodyMedium),
          ),
        ],
      ),
    );
  }
}

class _PkgFileTile extends StatelessWidget {
  final String pkgPath;
  final Future<PkgPreview> previewFuture;
  final void Function(String texPath) onTexTap;

  const _PkgFileTile({
    required this.pkgPath,
    required this.previewFuture,
    required this.onTexTap,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final fileName = pkgPath.split('/').last;

    return FutureBuilder<PkgPreview>(
      future: previewFuture,
      builder: (ctx, snap) {
        if (snap.connectionState != ConnectionState.done) {
          return ListTile(
            leading: const Icon(Icons.inventory_2),
            title: Text(fileName),
            subtitle: const Text('加载中...'),
            trailing: const SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
          );
        }
        if (snap.hasError) {
          return ListTile(
            leading: Icon(Icons.inventory_2, color: theme.colorScheme.error),
            title: Text(fileName),
            subtitle: Text(
              '预览失败: ${snap.error}',
              style: TextStyle(color: theme.colorScheme.error),
            ),
          );
        }

        final preview = snap.data!;
        return ExpansionTile(
          leading: const Icon(Icons.inventory_2),
          title: Text(fileName),
          subtitle: Text(
            '${preview.fileCount} 文件 · ${preview.texCount} TEX · v${preview.version}',
          ),
          children: preview.files.map((file) {
            return ListTile(
              contentPadding: const EdgeInsets.only(left: 72, right: 16),
              leading: Icon(
                file.isTex ? Icons.texture : Icons.insert_drive_file,
                size: 18,
                color: file.isTex ? Colors.blue : Colors.grey,
              ),
              title: Text(file.name, style: theme.textTheme.bodySmall),
              trailing: Text(
                file.formattedSize,
                style: theme.textTheme.labelSmall,
              ),
              onTap: file.isTex
                  ? () {
                      // 构造 TEX 文件的完整路径
                      final dir = pkgPath.substring(
                        0,
                        pkgPath.lastIndexOf('/'),
                      );
                      onTexTap('$dir/${file.name}');
                    }
                  : null,
            );
          }).toList(),
        );
      },
    );
  }
}

class _TexInfoRow extends StatelessWidget {
  final String label;
  final String value;
  const _TexInfoRow(this.label, this.value);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 4),
      child: Row(
        children: [
          SizedBox(
            width: 80,
            child: Text(
              label,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Theme.of(context).colorScheme.outline,
              ),
            ),
          ),
          Text(value, style: Theme.of(context).textTheme.bodyMedium),
        ],
      ),
    );
  }
}
