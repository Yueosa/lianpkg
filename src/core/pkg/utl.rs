pub(super) struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub(super) fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub(super) fn position(&self) -> usize {
        self.pos
    }

    pub(super) fn read_u32(&mut self) -> u32 {
        if self.pos + 4 > self.buf.len() {
            return 0;
        }
        let v = u32::from_le_bytes(
            self.buf[self.pos..self.pos + 4]
                .try_into()
                .unwrap(),
        );
        self.pos += 4;
        v
    }

    pub(super) fn read_string(&mut self) -> String {
        let len = self.read_u32() as usize;
        if self.pos + len > self.buf.len() {
            return String::new();
        }
        let s = String::from_utf8(
            self.buf[self.pos..self.pos + len].to_vec(),
        )
        .unwrap_or_else(|_| "<invalid utf8>".to_string());
        self.pos += len;
        s
    }
}
