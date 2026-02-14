//! 内部工具函数（不对外导出）

use crate::core::error::{CoreError, CoreResult};

/// 二进制数据读取器
pub(crate) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// 创建新的读取器
    pub(crate) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    /// 获取当前读取位置
    pub(crate) fn position(&self) -> usize {
        self.pos
    }

    /// 读取 u32（小端序）
    ///
    /// 越界时返回 `CoreError::InvalidData`
    pub(crate) fn read_u32(&mut self) -> CoreResult<u32> {
        if self.pos + 4 > self.buf.len() {
            return Err(CoreError::invalid_data(format!(
                "read_u32: need 4 bytes at offset {}, but buffer length is {}",
                self.pos,
                self.buf.len()
            )));
        }
        let v = u32::from_le_bytes(
            self.buf[self.pos..self.pos + 4]
                .try_into()
                .unwrap(),
        );
        self.pos += 4;
        Ok(v)
    }

    /// 读取字符串（长度前缀 u32 + UTF-8 内容）
    ///
    /// 越界或非法 UTF-8 时返回 `CoreError::InvalidData`
    pub(crate) fn read_string(&mut self) -> CoreResult<String> {
        let len = self.read_u32()? as usize;
        if self.pos + len > self.buf.len() {
            return Err(CoreError::invalid_data(format!(
                "read_string: need {} bytes at offset {}, but buffer length is {}",
                len, self.pos, self.buf.len()
            )));
        }
        let s = String::from_utf8(
            self.buf[self.pos..self.pos + len].to_vec(),
        )
        .map_err(|e| CoreError::invalid_data(format!(
            "read_string: invalid UTF-8 at offset {}: {}", self.pos, e
        )))?;
        self.pos += len;
        Ok(s)
    }
}
