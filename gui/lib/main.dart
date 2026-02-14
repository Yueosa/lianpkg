import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'widgets/app_shell.dart';

void main() {
  runApp(const ProviderScope(child: LianPkgApp()));
}

/// 粉蓝白主题色
const _kPrimarySeed = Color(0xFFE8839B); // 柔粉
const _kSecondarySeed = Color(0xFF7EB8D8); // 浅蓝

class LianPkgApp extends StatelessWidget {
  const LianPkgApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'LianPkg GUI',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: _kPrimarySeed,
          secondary: _kSecondarySeed,
          brightness: Brightness.light,
          surface: const Color(0xFFFCF8FA),
        ),
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: _kPrimarySeed,
          secondary: _kSecondarySeed,
          brightness: Brightness.dark,
          surface: const Color(0xFF1A1520),
        ),
        useMaterial3: true,
      ),
      themeMode: ThemeMode.system,
      home: const AppShell(),
    );
  }
}
