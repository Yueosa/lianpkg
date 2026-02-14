# core::pkg 模块

> PKG 文件解析与解包。支持预览（只读元数据）和完整解包两种模式。

## 文件结构

```
src/core/pkg/
├── mod.rs       # 模块声明与导出
├── structs.rs   # Input/Output + 运行时结构体
├── utl.rs       # Reader 二进制读取器
├── parse.rs     # 解析接口 (parse_pkg, parse_pkg_data)
└── unpack.rs    # 解包接口 (unpack_pkg, unpack_entry)
```

## 3 个公开接口

| 函数 | 说明 |
|------|------|
| `parse_pkg(ParsePkgInput)` | 解析 PKG 文件，返回元数据（不写磁盘） |
| `unpack_pkg(UnpackPkgInput)` | 解析 + 解包所有条目到输出目录 |
| `unpack_entry(UnpackEntryInput)` | 解包单个条目（精细控制） |

---

## PKG 文件格式

```
┌──────────────────────────────────────────┐
│ version_string (u32 长度前缀 + UTF-8)    │
│ file_count     (u32 LE)                  │
│ ┌────────────────────────────────────┐   │
│ │ entry[0].name   (u32 len + UTF-8) │   │
│ │ entry[0].offset (u32 LE)          │   │  ← 头部区
│ │ entry[0].size   (u32 LE)          │   │
│ ├────────────────────────────────────┤   │
│ │ entry[1] ...                       │   │
│ └────────────────────────────────────┘   │
│ ─ ─ ─ data_start ─ ─ ─ ─ ─ ─ ─ ─ ─ ─   │
│ [entry[0] 的原始数据]                     │
│ [entry[1] 的原始数据]                     │  ← 数据区
│ ...                                       │
└──────────────────────────────────────────┘
```

- `offset` 是相对于 data_start 的偏移
- 绝对位置 = `data_start + offset`

---

## Reader (`utl.rs`)

内部二进制读取器，**所有方法返回 `CoreResult`**：

```rust
impl Reader<'a> {
    fn read_u32(&mut self) -> CoreResult<u32>
    fn read_string(&mut self) -> CoreResult<String>
    fn position(&self) -> usize
}
```

### 错误处理

| 场景 | 旧行为 | 新行为 |
|------|--------|--------|
| `read_u32` 越界 | 静默返回 `0` | 返回 `CoreError::InvalidData`，含 offset 和 buffer 长度 |
| `read_string` 越界 | 静默返回空字符串 | 返回 `CoreError::InvalidData` |
| `read_string` 非法 UTF-8 | 返回 `"<invalid utf8>"` | 返回 `CoreError::InvalidData`，含具体 UTF-8 错误信息 |

---

## 解析校验 (`parse.rs`)

`parse_pkg_data()` 在解析过程中执行三层校验：

1. **读取校验** — Reader 的每次 `read_u32()` / `read_string()` 都通过 `?` 传播越界错误
2. **合理性检查** — `file_count > 100,000` 时立即拒绝，防止恶意文件触发巨量内存分配
3. **边界完整性** — 解析完所有 entry 后，验证每个 `data_start + offset + size ≤ data.len()`

任一校验失败均返回 `CoreError::InvalidData`，包含详细上下文信息。

---

## 零拷贝解包 (`unpack.rs`)

### 旧实现的问题

```rust
// 旧: 循环内每次 clone 整个 PKG 数据
for entry in &pkg_info.entries {
    unpack_entry(UnpackEntryInput {
        pkg_data: data.clone(),  // ← N 个 entry = N 次 full clone
        ...
    });
}
```

N 个条目 × PKG 文件大小 = O(N × filesize) 内存峰值。

### 新实现

```rust
// 新: 内部使用 &data 切片，零拷贝
for entry in &pkg_info.entries {
    write_entry(&data, data_start, entry, &output_path)?;
}
```

- `unpack_pkg` 调用内部 `write_entry(&[u8], ...)` 直接切片写出
- `unpack_entry`（公开接口）保持签名不变，内部委托给 `write_entry`
- 内存开销：O(filesize)，无论条目数量

---

## 结构体

```rust
// 解析结果
pub struct PkgInfo {
    pub version: String,
    pub file_count: u32,
    pub entries: Vec<PkgEntry>,
    pub data_start: usize,
}

pub struct PkgEntry {
    pub name: String,
    pub offset: u32,    // 相对于 data_start
    pub size: u32,
}

// 解包结果
pub struct ExtractedFile {
    pub entry_name: String,
    pub output_path: PathBuf,
    pub size: u32,
}
```

## 对外接口无破坏性变更

`parse_pkg` / `unpack_pkg` / `unpack_entry` 的签名和返回类型均未改变。  
`api/native/pkg.rs` 和 `cli/handlers/pkg.rs` 无需任何修改。
