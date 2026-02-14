/// 主导航 Shell — Material 3 NavigationRail + 内容区
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../pages/browser_page.dart';
import '../pages/dashboard_page.dart';
import '../pages/pipeline_page.dart';
import '../pages/settings_page.dart';
import '../pages/state_page.dart';
import '../providers/providers.dart';

class AppShell extends ConsumerWidget {
  const AppShell({super.key});

  static const _destinations = <NavigationRailDestination>[
    NavigationRailDestination(
      icon: Icon(Icons.dashboard_outlined),
      selectedIcon: Icon(Icons.dashboard),
      label: Text('总览'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.photo_library_outlined),
      selectedIcon: Icon(Icons.photo_library),
      label: Text('浏览'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.rocket_launch_outlined),
      selectedIcon: Icon(Icons.rocket_launch),
      label: Text('流水线'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.list_alt_outlined),
      selectedIcon: Icon(Icons.list_alt),
      label: Text('状态'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.settings_outlined),
      selectedIcon: Icon(Icons.settings),
      label: Text('设置'),
    ),
  ];

  static const _pages = <Widget>[
    DashboardPage(),
    BrowserPage(),
    PipelinePage(),
    StatePage(),
    SettingsPage(),
  ];

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final index = ref.watch(navigationIndexProvider);
    final theme = Theme.of(context);

    return Scaffold(
      body: Row(
        children: [
          NavigationRail(
            selectedIndex: index,
            onDestinationSelected: (i) {
              final prev = ref.read(navigationIndexProvider);
              ref.read(navigationIndexProvider.notifier).state = i;
              // 切换到 tab 时刷新该 tab 需要的数据
              if (i != prev) {
                if (i == 0) {
                  ref.invalidate(statusProvider);
                  ref.invalidate(stateProvider);
                } else if (i == 1) {
                  ref.invalidate(scanResultProvider);
                } else if (i == 2) {
                  ref.invalidate(configProvider);
                }
              }
            },
            labelType: NavigationRailLabelType.all,
            destinations: _destinations,
            leading: Padding(
              padding: const EdgeInsets.symmetric(vertical: 16),
              child: Column(
                children: [
                  ClipRRect(
                    borderRadius: BorderRadius.circular(8),
                    child: Image.asset(
                      'assets/icon.png',
                      width: 32,
                      height: 32,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    'LianPkg',
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: theme.colorScheme.primary,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
            ),
          ),
          const VerticalDivider(thickness: 1, width: 1),
          Expanded(
            child: AnimatedSwitcher(
              duration: const Duration(milliseconds: 200),
              child: _pages[index],
            ),
          ),
        ],
      ),
    );
  }
}
