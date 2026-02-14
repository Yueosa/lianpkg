# core::error 模块

统一错误处理模块，为 `core` 层所有子模块提供错误类型和 `Result` 别名。

## CoreError

```rust
pub enum CoreError {
    Io { message, path? },
    Parse { message, source? },
    InvalidData { context },
    Validation { message },
    NotFound { message, path? },
    Unsupported { message },
}
```

### 变体说明

| 变体 | 用途 | 示例场景 |
|------|------|---------|
| `Io` | 文件/目录 I/O 操作失败 | 读写文件失败、权限不足、磁盘满 |
| `Parse` | 格式解析失败 | TOML/JSON/VDF 语法错误 |
| `InvalidData` | 二进制数据校验失败 | PKG magic bytes 不匹配、TEX 偏移越界、file_count 超出合理范围 |
| `Validation` | 参数/配置验证失败 | 配置项值非法、路径格式错误 |
| `NotFound` | 文件/目录/条目不存在 | Workshop 路径不存在、config.toml 未找到 |
| `Unsupported` | 不支持的格式或版本 | 未知 TEX format ID、不支持的 TEX 版本号 |

### InvalidData vs Parse

- `Parse`：**文本格式**解析失败（TOML 语法错误、JSON 格式不合法）
- `InvalidData`：**二进制数据**校验失败（magic bytes 错误、偏移越界、字段值不在合法范围）

## CoreResult

```rust
pub type CoreResult<T> = Result<T, CoreError>;
```

所有 `core` 子模块的公开函数都应返回 `CoreResult<T>`。

## 便捷构造函数

每个变体都有对应的构造函数，参数接受 `impl Into<String>`：

```rust
CoreError::io("read failed")
CoreError::io_with_path("read failed", "/path/to/file")
CoreError::parse("unexpected token")
CoreError::parse_with_source("unexpected token", "TOML")
CoreError::invalid_data("PKG magic bytes mismatch: expected 0x4B505247")
CoreError::validation("format must be png or jpg")
CoreError::not_found("file does not exist")
CoreError::not_found_with_path("file does not exist", "/path/to/file")
CoreError::unsupported("unknown TEX format: 0xFF")
```

**约定：** 统一使用便捷构造函数，不要直接构造 `CoreError::Variant { .. }` 结构体。

## From 自动转换

支持通过 `?` 操作符自动转换以下错误类型：

| 源类型 | 转换为 | 说明 |
|--------|--------|------|
| `std::io::Error` | `CoreError::Io` | 标准 I/O 错误 |
| `toml::de::Error` | `CoreError::Parse { source: "TOML" }` | TOML 解析错误 |
| `serde_json::Error` | `CoreError::Parse { source: "JSON" }` | JSON 解析错误 |
| `image::ImageError` | `CoreError::Io` | 图片编解码/保存错误 |

## 设计决策

- **不派生 `Serialize` / `Deserialize`**：错误类型仅用于内部传递，不需要序列化。api 层向外输出时应将 `CoreError` 转换为自定义响应格式。
- **`Clone` 保留**：部分场景需要克隆错误（如在日志和返回值中同时使用）。
- **便捷构造函数强制约定**：避免跨模块构造风格不一致（部分用结构体字面量、部分用函数）。
