# error 模块

## 概述

- **模块职责**: 提供统一的错误类型和 Result 别名，确保 core 模块 API 的一致性
- **使用场景**: 所有 core 子模块（path、cfg、paper、pkg、tex）的错误处理
- **依赖关系**: 被所有 core 子模块依赖，是最底层的基础模块

## 接口列表

| 类型 | 名称            | 说明                        |
| ---- | --------------- | --------------------------- |
| enum | `CoreError`     | 核心错误枚举，5 个变体      |
| type | `CoreResult<T>` | `Result<T, CoreError>` 别名 |

## 数据结构

### CoreError 枚举

```rust
pub enum CoreError {
    /// 文件/目录 I/O 错误
    Io {
        message: String,
        path: Option<String>,
    },
    /// 解析错误 (TOML/JSON/PKG/TEX)
    Parse {
        message: String,
        source: Option<String>,
    },
    /// 验证错误 (参数无效、格式不符等)
    Validation { message: String },
    /// 未找到 (文件/目录/条目不存在)
    NotFound {
        message: String,
        path: Option<String>,
    },
    /// 格式不支持 (未知的文件格式等)
    Unsupported { message: String },
}
```

### CoreResult 类型别名

```rust
pub type CoreResult<T> = Result<T, CoreError>;
```

## 错误变体详解

### `Io` - I/O 错误

| 字段      | 类型             | 说明                 |
| --------- | ---------------- | -------------------- |
| `message` | `String`         | 错误描述             |
| `path`    | `Option<String>` | 相关文件路径（可选） |

**使用场景**:
- 文件读写失败
- 目录创建/删除失败
- 权限不足

### `Parse` - 解析错误

| 字段      | 类型             | 说明                                 |
| --------- | ---------------- | ------------------------------------ |
| `message` | `String`         | 错误描述                             |
| `source`  | `Option<String>` | 数据来源（如 "TOML"、"JSON"、"PKG"） |

**使用场景**:
- TOML/JSON 格式错误
- PKG/TEX 文件头解析失败
- 二进制数据解析异常

### `Validation` - 验证错误

| 字段      | 类型     | 说明     |
| --------- | -------- | -------- |
| `message` | `String` | 错误描述 |

**使用场景**:
- 参数校验失败
- 路径格式无效
- 配置项不合法

### `NotFound` - 未找到

| 字段      | 类型             | 说明             |
| --------- | ---------------- | ---------------- |
| `message` | `String`         | 错误描述         |
| `path`    | `Option<String>` | 目标路径（可选） |

**使用场景**:
- 文件/目录不存在
- PKG 条目不存在
- 配置文件缺失

### `Unsupported` - 不支持

| 字段      | 类型     | 说明     |
| --------- | -------- | -------- |
| `message` | `String` | 错误描述 |

**使用场景**:
- 不支持的 TEX 格式
- 未知的 PKG 版本
- 平台不支持的操作

## 便捷构造函数

```rust
impl CoreError {
    // 基础构造
    pub fn io(msg: impl Into<String>) -> Self;
    pub fn parse(msg: impl Into<String>) -> Self;
    pub fn validation(msg: impl Into<String>) -> Self;
    pub fn not_found(msg: impl Into<String>) -> Self;
    pub fn unsupported(msg: impl Into<String>) -> Self;

    // 带附加信息的构造
    pub fn io_with_path(msg: impl Into<String>, path: impl Into<String>) -> Self;
    pub fn parse_with_source(msg: impl Into<String>, source: impl Into<String>) -> Self;
    pub fn not_found_with_path(msg: impl Into<String>, path: impl Into<String>) -> Self;
}
```

## From Trait 实现

自动从标准库错误类型转换：

| 源类型              | 目标变体 | source 值 |
| ------------------- | -------- | --------- |
| `std::io::Error`    | `Io`     | -         |
| `toml::de::Error`   | `Parse`  | "TOML"    |
| `serde_json::Error` | `Parse`  | "JSON"    |

## 使用示例

### 基础错误创建

```rust
use crate::core::error::{CoreError, CoreResult};

fn example() -> CoreResult<()> {
    // 创建简单错误
    return Err(CoreError::validation("路径不能为空"));

    // 创建带路径的错误
    return Err(CoreError::io_with_path("读取失败", "/path/to/file"));

    // 创建带来源的解析错误
    return Err(CoreError::parse_with_source("无效的格式", "TEX"));
}
```

### 使用 ? 操作符自动转换

```rust
use std::fs;
use crate::core::error::CoreResult;

fn read_config() -> CoreResult<String> {
    // std::io::Error 自动转换为 CoreError::Io
    let content = fs::read_to_string("config.toml")?;
    Ok(content)
}
```

### 错误匹配

```rust
use crate::core::error::CoreError;

fn handle_error(err: CoreError) {
    match err {
        CoreError::Io { message, path } => {
            if let Some(p) = path {
                eprintln!("IO 错误 [{}]: {}", p, message);
            } else {
                eprintln!("IO 错误: {}", message);
            }
        }
        CoreError::NotFound { message, path } => {
            eprintln!("未找到: {}", message);
        }
        CoreError::Unsupported { message } => {
            eprintln!("不支持: {}", message);
        }
        _ => eprintln!("其他错误: {}", err),
    }
}
```

## 设计说明

### 为什么使用枚举而非 trait object?

1. **可序列化**: `CoreError` 实现 `Serialize`/`Deserialize`，便于 FFI 跨语言传递
2. **可 Clone**: 枚举可以 `Clone`，而 `Box<dyn Error>` 不行
3. **模式匹配**: 调用方可以精确匹配错误类型，做针对性处理
4. **性能**: 无堆分配（除了 String 字段），无动态分发

### 与 std::error::Error 的关系

`CoreError` 实现了 `std::error::Error` trait，可以：
- 使用 `?` 传播
- 作为 `Box<dyn Error>` 使用（如需要）
- 用 `.to_string()` 获取格式化消息
