/// Settings 配置管理页 — 可视化编辑 config.toml
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../providers/providers.dart';
import '../utils/open_folder.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage> {
  bool _saving = false;

  Future<void> _setConfig(String key, String value) async {
    setState(() => _saving = true);
    try {
      await ref.read(lianpkgServiceProvider).setConfig(key, value);
      ref.invalidate(configProvider);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('已保存: $key'),
            duration: const Duration(seconds: 1),
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('保存失败: $e'), backgroundColor: Colors.red),
        );
      }
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  Future<void> _resetConfig() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('重置配置'),
        content: const Text('确定要将所有配置恢复为默认值吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('重置'),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    try {
      await ref.read(lianpkgServiceProvider).resetConfig();
      ref.invalidate(configProvider);
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('配置已重置')));
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('重置失败: $e'), backgroundColor: Colors.red),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final configAsync = ref.watch(configProvider);
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.all(24),
      child: configAsync.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, _) => Center(child: Text('加载失败: $e')),
        data: (config) => ListView(
          children: [
            Row(
              children: [
                Text('设置', style: theme.textTheme.headlineMedium),
                const Spacer(),
                if (_saving)
                  const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
                const SizedBox(width: 12),
                OutlinedButton.icon(
                  onPressed: _resetConfig,
                  icon: const Icon(Icons.restart_alt),
                  label: const Text('重置为默认'),
                ),
              ],
            ),
            const SizedBox(height: 24),

            // 路径配置
            Text('路径', style: theme.textTheme.titleLarge),
            const SizedBox(height: 12),
            _PathField(
              label: 'Workshop 路径',
              value: config.workshopPath,
              onSave: (v) => _setConfig('wallpaper.workshop_path', v),
            ),
            _PathField(
              label: 'Raw 输出路径',
              value: config.rawOutputPath,
              onSave: (v) => _setConfig('wallpaper.raw_output_path', v),
            ),
            _PathField(
              label: '解包输出路径',
              value: config.unpackedOutputPath,
              onSave: (v) => _setConfig('unpack.unpacked_output_path', v),
            ),
            _PathField(
              label: '转换输出路径',
              value: config.convertedOutputPath,
              onSave: (v) => _setConfig('tex.converted_output_path', v),
            ),
            const SizedBox(height: 24),

            // 开关配置
            Text('选项', style: theme.textTheme.titleLarge),
            const SizedBox(height: 12),
            SwitchListTile(
              title: const Text('启用 Raw 输出'),
              subtitle: const Text('直接复制非 PKG 壁纸到输出目录'),
              value: config.enableRawOutput,
              onChanged: (v) =>
                  _setConfig('wallpaper.enable_raw_output', v.toString()),
            ),
            SwitchListTile(
              title: const Text('清理解包中间文件'),
              subtitle: const Text('转换 TEX 后删除解包的中间文件'),
              value: config.cleanUnpacked,
              onChanged: (v) =>
                  _setConfig('unpack.clean_unpacked', v.toString()),
            ),
            const Divider(),

            // 流水线配置
            Text('流水线', style: theme.textTheme.titleLarge),
            const SizedBox(height: 12),
            SwitchListTile(
              title: const Text('增量处理'),
              subtitle: const Text('跳过已处理的壁纸'),
              value: config.pipeline.incremental,
              onChanged: (v) =>
                  _setConfig('pipeline.incremental', v.toString()),
            ),
            SwitchListTile(
              title: const Text('自动解包 PKG'),
              subtitle: const Text('自动解包壁纸中的 PKG 文件'),
              value: config.pipeline.autoUnpackPkg,
              onChanged: (v) =>
                  _setConfig('pipeline.auto_unpack_pkg', v.toString()),
            ),
            SwitchListTile(
              title: const Text('自动转换 TEX'),
              subtitle: const Text('自动将 TEX 纹理转换为 PNG 图片'),
              value: config.pipeline.autoConvertTex,
              onChanged: (v) =>
                  _setConfig('pipeline.auto_convert_tex', v.toString()),
            ),
          ],
        ),
      ),
    );
  }
}

class _PathField extends StatefulWidget {
  final String label;
  final String value;
  final ValueChanged<String> onSave;

  const _PathField({
    required this.label,
    required this.value,
    required this.onSave,
  });

  @override
  State<_PathField> createState() => _PathFieldState();
}

class _PathFieldState extends State<_PathField> {
  late final TextEditingController _controller;
  bool _dirty = false;

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.value);
  }

  @override
  void didUpdateWidget(_PathField oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value && !_dirty) {
      _controller.text = widget.value;
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: TextField(
        controller: _controller,
        decoration: InputDecoration(
          labelText: widget.label,
          border: const OutlineInputBorder(),
          suffixIcon: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              IconButton(
                icon: const Icon(Icons.folder_open),
                tooltip: '在文件管理器中打开',
                onPressed: () async {
                  final path = _controller.text;
                  final ok = await openFolder(path);
                  if (!ok && context.mounted) {
                    ScaffoldMessenger.of(
                      context,
                    ).showSnackBar(SnackBar(content: Text('无法打开: $path')));
                  }
                },
              ),
              if (_dirty)
                IconButton(
                  icon: const Icon(Icons.save),
                  onPressed: () {
                    widget.onSave(_controller.text);
                    setState(() => _dirty = false);
                  },
                ),
            ],
          ),
        ),
        onChanged: (v) {
          if (!_dirty) setState(() => _dirty = true);
        },
        onSubmitted: (v) {
          widget.onSave(v);
          setState(() => _dirty = false);
        },
      ),
    );
  }
}
