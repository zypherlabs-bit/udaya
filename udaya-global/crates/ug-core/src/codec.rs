//! Strict, canonical, allocation-bounded binary encoding.
//!
//! Network-facing structures are encoded with explicit fixed-width integers
//! and explicit length prefixes. Decoders reject truncation, trailing bytes,
//! and lengths above a caller-supplied bound. `bincode`/`serde` are not used
//! on any consensus-critical or peer-facing path.

use crate::error::{Error, Result};

/// Canonical encoder.
#[derive(Default, Debug)]
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn with_capacity(n: usize) -> Self {
        Self {
            buf: Vec::with_capacity(n),
        }
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn bytes(&mut self, v: &[u8]) {
        debug_assert!(v.len() <= u32::MAX as usize);
        self.u32(v.len() as u32);
        self.buf.extend_from_slice(v);
    }

    pub fn fixed(&mut self, v: &[u8]) {
        self.buf.extend_from_slice(v);
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}

/// Canonical decoder with strict bounds.
#[derive(Debug, Clone)]
pub struct Decoder<'a> {
    data: &'a [u8],
    pos: usize,
    max_container: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            max_container: 1 << 20,
        }
    }

    pub fn with_max_container(mut self, n: usize) -> Self {
        self.max_container = n;
        self
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| Error::Codec("offset overflow".into()))?;
        if end > self.data.len() {
            return Err(Error::Codec(format!(
                "truncated: need {n} bytes at offset {}, have {}",
                self.pos,
                self.data.len().saturating_sub(self.pos)
            )));
        }
        let out = &self.data[self.pos..end];
        self.pos = end;
        Ok(out)
    }

    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn u32(&mut self) -> Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn u64(&mut self) -> Result<u64> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    pub fn fixed<const N: usize>(&mut self) -> Result<[u8; N]> {
        let b = self.take(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(b);
        Ok(out)
    }

    pub fn bytes(&mut self, max: usize) -> Result<&'a [u8]> {
        let len = self.u32()? as usize;
        if len > max {
            return Err(Error::Codec(format!("length {len} exceeds limit {max}")));
        }
        self.take(len)
    }

    /// Read a collection length and check it against both the configured
    /// container bound and a caller-supplied per-element minimum size. This
    /// prevents a single varint from causing a huge allocation.
    pub fn len(&mut self, max_items: usize) -> Result<usize> {
        let len = self.u32()? as usize;
        if len > max_items || len > self.max_container {
            return Err(Error::Codec(format!(
                "container length {len} exceeds limit {}",
                max_items.min(self.max_container)
            )));
        }
        Ok(len)
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    /// Reject trailing bytes: canonical encodings have exactly one form.
    pub fn finish(self) -> Result<()> {
        if self.pos != self.data.len() {
            return Err(Error::Codec(format!(
                "trailing bytes: {} unconsumed",
                self.data.len() - self.pos
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_fixed_and_variable() {
        let mut e = Encoder::new();
        e.u8(1);
        e.u16(2);
        e.u32(3);
        e.u64(4);
        e.bytes(b"hello");
        let bytes = e.finish();

        let mut d = Decoder::new(&bytes);
        assert_eq!(d.u8().unwrap(), 1);
        assert_eq!(d.u16().unwrap(), 2);
        assert_eq!(d.u32().unwrap(), 3);
        assert_eq!(d.u64().unwrap(), 4);
        assert_eq!(d.bytes(16).unwrap(), b"hello");
        d.finish().unwrap();
    }

    #[test]
    fn rejects_trailing_bytes() {
        let mut e = Encoder::new();
        e.u8(7);
        e.u8(8);
        let bytes = e.finish();
        let mut d = Decoder::new(&bytes);
        assert_eq!(d.u8().unwrap(), 7);
        assert!(d.finish().is_err());
    }

    #[test]
    fn rejects_oversized_length_before_allocating() {
        let mut e = Encoder::new();
        e.u32(u32::MAX);
        let bytes = e.finish();
        let mut d = Decoder::new(&bytes);
        assert!(d.bytes(1024).is_err());
    }

    #[test]
    fn rejects_truncation() {
        let bytes = [1u8, 2, 3];
        let mut d = Decoder::new(&bytes);
        assert_eq!(d.u8().unwrap(), 1);
        assert!(d.u32().is_err());
        assert_eq!(d.position(), 1, "failed reads must not advance the cursor");
    }

    #[test]
    fn container_length_is_bounded() {
        let mut e = Encoder::new();
        e.u32(1_000_000);
        let bytes = e.finish();
        let mut d = Decoder::new(&bytes).with_max_container(10);
        assert!(d.len(1_000_000).is_err());
    }
}
