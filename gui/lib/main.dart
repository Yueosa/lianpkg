import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'widgets/app_shell.dart';

void main() {
  runApp(const ProviderScope(child: LianPkgApp()));
}

class LianPkgApp extends StatelessWidget {
  const LianPkgApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'LianPkg',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorSchemeSeed: const Color(0xFF6750A4),
        useMaterial3: true,
        brightness: Brightness.light,
      ),
      darkTheme: ThemeData(
        colorSchemeSeed: const Color(0xFF6750A4),
        useMaterial3: true,
        brightness: Brightness.dark,
      ),
      themeMode: ThemeMode.system,
      home: const AppShell(),
    );
  }
}
