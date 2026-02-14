# LianPkg FFI 集成文档

> 面向希望在自己的应用中集成 LianPkg 核心功能的开发者。  
> LianPkg 对外提供 C ABI 兼容的共享库（`.so` / `.dll`），通过 JSON 协议通信。

---

## 概述

LianPkg FFI 层仅暴露两个 C 函数：

```c
// 主入口：JSON 请求 → JSON 响应
char* lianpkg_call(const char* json_input);

// 释放 lianpkg_call 返回的字符串
void lianpkg_free_string(char* s);
```

所有功能通过 `lianpkg_call` 的 JSON `action` 字段分发，无复杂类型绑定，任何支持 C FFI 的语言均可调用。

---

## 共享库文件

| 平台 | 文件名 | 编译命令 |
|------|--------|---------|
| Linux | `liblianpkg.so` | `cargo build --release --lib` |
| Windows | `lianpkg.dll` | `cargo build --release --lib --target x86_64-pc-windows-gnu` |

### 编译选项

LianPkg 使用以下 release profile，输出极小体积：

```toml
[profile.release]
opt-level = "z"       # 最小体积优化
lto = true            # 链接时优化
codegen-units = 1     # 单编译单元
strip = true          # 剥离调试符号
panic = "abort"       # panic 直接终止（不解栈）
```

---

## 通信协议

### 请求格式

```json
{
  "action": "动作名称",
  "params": { ... }
}
```

### 响应格式

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

失败时：

```json
{
  "success": false,
  "data": null,
  "error": "错误描述"
}
```

---

## 支持的 Action

### init — 初始化上下文

必须在其他操作之前调用。创建/加载配置文件和状态文件。

**请求**

```json
{ "action": "init", "params": { "config_dir": null } }
```

| 参数 | 类型 | 说明 |
|------|------|------|
| `config_dir` | `string?` | 自定义配置目录，null 使用默认路径 |

**响应 data**：`AppContext`（含 `config`、`config_path`、`state_path`）

---

### scan — 扫描壁纸

扫描 Workshop 目录，返回壁纸列表和统计。

**请求**

```json
{ "action": "scan", "params": { "workshop_path": null } }
```

| 参数 | 类型 | 说明 |
|------|------|------|
| `workshop_path` | `string?` | 自定义 Workshop 路径，null 使用配置中的路径 |

**响应 data**：`ScanOutput`

```json
{
  "wallpapers": [
    {
      "wallpaper_id": "123456789",
      "title": "星空少女",
      "wallpaper_type": "scene",
      "has_pkg": true,
      "pkg_files": ["/path/to/scene.pkg"],
      "folder_path": "/path/to/123456789",
      "preview_path": "/path/to/preview.jpg",
      "is_processed": false,
      "tags": ["Anime", "Girls"]
    }
  ],
  "stats": {
    "total_count": 150,
    "pkg_count": 120,
    "raw_count": 30,
    "processed_count": 80,
    "pending_count": 70
  }
}
```

---

### auto — 执行自动流水线

按顺序执行：扫描 → 复制 → 解包 PKG → 转换 TEX → 清理。

**请求**

```json
{
  "action": "auto",
  "params": {
    "wallpaper_ids": null,
    "no_raw": false,
    "no_tex": false,
    "no_clean_unpacked": false,
    "no_incremental": false
  }
}
```

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `wallpaper_ids` | `string[]?` | `null` | 只处理指定 ID，null 为全部 |
| `no_raw` | `bool` | `false` | 跳过原始壁纸复制 |
| `no_tex` | `bool` | `false` | 跳过 TEX 转换 |
| `no_clean_unpacked` | `bool` | `false` | 保留解包中间产物 |
| `no_incremental` | `bool` | `false` | 禁用增量处理 |

**响应 data**：`AutoOutput`（含 `copy_output`、`pkg_output`、`tex_output`、`stats`）

> **注意**: `auto` 是阻塞调用，处理大量壁纸时应在后台线程执行。

---

### progress — 轮询流水线进度

在 `auto` 执行期间，从另一个线程调用此 action 获取实时进度。

**请求**

```json
{ "action": "progress", "params": {} }
```

**响应 data**：`ProgressSnapshot`

```json
{
  "running": true,
  "percent": 65,
  "stage": "converting",
  "message": "Converting TEX files...",
  "current_item": "materials/wall.tex"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `running` | `bool` | 流水线是否在运行 |
| `percent` | `int` | 当前进度百分比 (0-100) |
| `stage` | `string` | 当前阶段：`init` / `scanning` / `copying` / `unpacking` / `converting` / `cleanup` / `done` |
| `message` | `string` | 人类可读消息 |
| `current_item` | `string?` | 当前处理项 |

---

### pkg_unpack — 解包 PKG 文件

**请求**

```json
{
  "action": "pkg_unpack",
  "params": {
    "sources": [
      {
        "wallpaper_id": "123456789",
        "pkg_paths": ["/path/to/scene.pkg"]
      }
    ],
    "output": "/path/to/unpacked"
  }
}
```

**响应 data**：`UnpackOutput`（含 `results` 列表 + `stats`）

---

### pkg_preview — 预览 PKG 内容

**请求**

```json
{ "action": "pkg_preview", "params": { "path": "/path/to/scene.pkg" } }
```

**响应 data**：`PkgPreview`

```json
{
  "version": "PKG0",
  "file_count": 12,
  "files": [
    { "name": "materials/wall.tex", "size": 65536, "is_tex": true },
    { "name": "scene.json", "size": 1024, "is_tex": false }
  ],
  "tex_count": 8
}
```

---

### tex_convert — 批量转换 TEX

**请求**

```json
{
  "action": "tex_convert",
  "params": {
    "input": "/path/to/unpacked",
    "output": "/path/to/output"
  }
}
```

| 参数 | 类型 | 说明 |
|------|------|------|
| `input` | `string` | 输入路径（含 .tex 文件的目录） |
| `output` | `string?` | 输出路径，null 使用配置默认值 |

**响应 data**：`ConvertOutput`（含 `results` 列表 + `stats`）

---

### tex_preview — 预览 TEX 元数据

**请求**

```json
{ "action": "tex_preview", "params": { "path": "/path/to/texture.tex" } }
```

**响应 data**：`TexPreview`

```json
{
  "version": "TEXB0003",
  "format": "DXT5",
  "width": 1024,
  "height": 1024,
  "image_count": 1,
  "mipmap_count": 10,
  "is_compressed": true,
  "is_video": false,
  "data_size": 524288,
  "recommended_output": "png"
}
```

---

### config_get — 获取配置

**请求**

```json
{ "action": "config_get", "params": {} }
```

**响应 data**：完整的 `RuntimeConfig` 对象，包含所有路径和开关配置。

---

### config_set — 设置配置项

**请求**

```json
{ "action": "config_set", "params": { "key": "wallpaper.workshop_path", "value": "/new/path" } }
```

| 参数 | 类型 | 说明 |
|------|------|------|
| `key` | `string` | 配置键（TOML 节.字段格式） |
| `value` | `string` | 新值 |

**可用的配置键**

| 键 | 类型 | 说明 |
|----|------|------|
| `wallpaper.workshop_path` | 路径 | Workshop 壁纸目录 |
| `wallpaper.raw_output_path` | 路径 | 原始壁纸输出路径 |
| `wallpaper.enable_raw_output` | bool | 是否复制非 PKG 壁纸 |
| `unpack.unpacked_output_path` | 路径 | PKG 解包输出路径 |
| `unpack.clean_unpacked` | bool | 转换后是否清理解包产物 |
| `tex.converted_output_path` | 路径 | TEX 转换输出路径 |
| `pipeline.incremental` | bool | 是否启用增量处理 |
| `pipeline.auto_unpack_pkg` | bool | 自动流水线是否解包 PKG |
| `pipeline.auto_convert_tex` | bool | 自动流水线是否转换 TEX |

---

### config_reset — 重置配置

删除并重建 `config.toml`，恢复默认值。

**请求**

```json
{ "action": "config_reset", "params": {} }
```

---

### state_get — 获取处理状态

**请求**

```json
{ "action": "state_get", "params": {} }
```

**响应 data**：`StateData`

```json
{
  "processed": {
    "123456789": {
      "title": "星空少女",
      "process_type": "Pkg",
      "processed_at": "2026-02-15T10:30:00+08:00",
      "output_path": "/home/user/.local/share/lianpkg/Pkg_Unpacked/123456789"
    }
  },
  "last_run": "2026-02-15T10:30:00+08:00"
}
```

---

### state_clear — 清空处理状态

删除并重建 `state.json`，下次运行将重新处理所有壁纸。

**请求**

```json
{ "action": "state_clear", "params": {} }
```

---

### status — 综合状态

返回合并的统计信息（需要同时扫描 Workshop 和读取 state）。

**请求**

```json
{ "action": "status", "params": {} }
```

**响应 data**

```json
{
  "total_wallpapers": 150,
  "total_processed": 80,
  "processed_pkg": 60,
  "processed_raw": 15,
  "processed_skipped": 5,
  "pending_total": 70,
  "pending_pkg": 55,
  "pending_raw": 15,
  "pending_pkg_size": 2147483648,
  "last_run": "2026-02-15T10:30:00+08:00",
  "disk_usage": {
    "raw_output_size": 1073741824,
    "unpacked_output_size": 536870912,
    "converted_output_size": 2147483648,
    "available_space": 53687091200
  }
}
```

---

## 调用流程

### 典型使用顺序

```
1. init          ← 必须首先调用
2. scan          ← 获取壁纸列表
3. auto          ← 执行流水线（后台线程）
   ├─ progress   ← 轮询进度（UI 线程）
   ├─ progress
   └─ progress
4. status        ← 获取处理结果
```

### 进度轮询模式

`auto` action 是一个阻塞操作。推荐的使用模式：

1. 在**后台线程**调用 `auto`
2. 在**UI 线程**定期调用 `progress`（建议间隔 200-500ms）
3. 当 `progress` 返回 `running: false` 时停止轮询
4. 取回 `auto` 的返回值

---

## 内存管理

**关键规则**：`lianpkg_call` 返回的指针指向 Rust 分配的内存，**必须**通过 `lianpkg_free_string` 释放。

```
调用方                                   Rust 侧
  │                                        │
  │──── lianpkg_call(json) ───────────────▶│
  │                                        │ 分配 CString
  │◀──── char* response ──────────────────│
  │                                        │
  │  使用 response ...                     │
  │                                        │
  │──── lianpkg_free_string(response) ────▶│
  │                                        │ 释放 CString
```

不调用 `lianpkg_free_string` 会导致内存泄漏。每个指针只能释放一次。

---

## Panic 安全

FFI 层使用 `panic::catch_unwind` 包裹所有逻辑，保证 Rust panic 不会跨 FFI 边界传播。发生 panic 时返回错误 JSON 响应。

---

## 各语言集成示例

### Dart (Flutter)

```dart
import 'dart:ffi';
import 'dart:convert';
import 'package:ffi/ffi.dart';

typedef LianpkgCallC = Pointer<Utf8> Function(Pointer<Utf8>);
typedef LianpkgCallDart = Pointer<Utf8> Function(Pointer<Utf8>);
typedef LianpkgFreeC = Void Function(Pointer<Utf8>);
typedef LianpkgFreeDart = void Function(Pointer<Utf8>);

final lib = DynamicLibrary.open('liblianpkg.so');
final call = lib.lookupFunction<LianpkgCallC, LianpkgCallDart>('lianpkg_call');
final free = lib.lookupFunction<LianpkgFreeC, LianpkgFreeDart>('lianpkg_free_string');

Map<String, dynamic> lianpkgCall(String action, [Map<String, dynamic> params = const {}]) {
  final json = jsonEncode({'action': action, 'params': params});
  final reqPtr = json.toNativeUtf8();
  final resPtr = call(reqPtr);
  final result = jsonDecode(resPtr.toDartString()) as Map<String, dynamic>;
  malloc.free(reqPtr);
  free(resPtr);
  return result;
}
```

### Python (ctypes)

```python
import ctypes
import json

lib = ctypes.CDLL('./liblianpkg.so')
lib.lianpkg_call.restype = ctypes.c_char_p
lib.lianpkg_call.argtypes = [ctypes.c_char_p]
lib.lianpkg_free_string.restype = None
lib.lianpkg_free_string.argtypes = [ctypes.c_char_p]

def lianpkg_call(action: str, params: dict = {}) -> dict:
    req = json.dumps({"action": action, "params": params}).encode("utf-8")
    res_ptr = lib.lianpkg_call(req)
    result = json.loads(res_ptr.decode("utf-8"))
    lib.lianpkg_free_string(res_ptr)
    return result

# 使用
result = lianpkg_call("init")
print(result)
```

### C

```c
#include <stdio.h>
#include <stdlib.h>

// 声明 FFI 函数
extern char* lianpkg_call(const char* json_input);
extern void lianpkg_free_string(char* s);

int main() {
    const char* request = "{\"action\":\"init\",\"params\":{}}";
    char* response = lianpkg_call(request);
    printf("Response: %s\n", response);
    lianpkg_free_string(response);
    return 0;
}
```

编译：
```bash
gcc main.c -L. -llianpkg -o main
LD_LIBRARY_PATH=. ./main
```

### Go (cgo)

```go
package main

/*
#cgo LDFLAGS: -L. -llianpkg
#include <stdlib.h>

extern char* lianpkg_call(const char* json_input);
extern void lianpkg_free_string(char* s);
*/
import "C"
import (
    "encoding/json"
    "fmt"
    "unsafe"
)

func lianpkgCall(action string, params map[string]any) map[string]any {
    req, _ := json.Marshal(map[string]any{"action": action, "params": params})
    cReq := C.CString(string(req))
    defer C.free(unsafe.Pointer(cReq))

    cRes := C.lianpkg_call(cReq)
    defer C.lianpkg_free_string(cRes)

    var result map[string]any
    json.Unmarshal([]byte(C.GoString(cRes)), &result)
    return result
}

func main() {
    result := lianpkgCall("init", map[string]any{})
    fmt.Println(result)
}
```

---

## 错误处理

所有错误通过 `success: false` + `error` 字段返回，不会出现以下情况：

- Rust panic 传播到调用方
- 返回 null 指针（失败时也返回有效的错误 JSON）
- 段错误（输入 null 指针也能优雅处理）

建议调用方始终检查 `success` 字段后再读取 `data`。

---

*最后更新于 2026-02-15，对应 LianPkg v2.0.1。*
