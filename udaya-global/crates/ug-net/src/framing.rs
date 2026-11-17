//! Bounded, length-prefixed framing.
//!
//! A frame is a 4-byte big-endian length followed by exactly that many bytes.
//! The length is validated *before* allocating or reading the body. Zero-length
//! frames are rejected.

use crate::{Error, Result};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const LENGTH_BYTES: usize = 4;

pub(crate) fn validate_frame_len(len: usize, max: usize) -> Result<()> {
    if len == 0 {
        return Err(Error::Codec("zero-length frame".into()));
    }
    if len > max {
        return Err(Error::FrameTooLarge(len));
    }
    Ok(())
}

pub(crate) async fn write_frame<W: AsyncWrite + Unpin>(
    writer: &mut W,
    data: &[u8],
    max: usize,
) -> Result<()> {
    validate_frame_len(data.len(), max)?;
    let len = u32::try_from(data.len()).map_err(|_| Error::FrameTooLarge(data.len()))?;
    writer.write_all(&len.to_be_bytes()).await?;
    writer.write_all(data).await?;
    writer.flush().await?;
    Ok(())
}

pub(crate) async fn read_frame<R: AsyncRead + Unpin>(
    reader: &mut R,
    max: usize,
) -> Result<Vec<u8>> {
    let mut len_bytes = [0u8; LENGTH_BYTES];
    reader.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    validate_frame_len(len, max)?;
    let mut data = vec![0u8; len];
    reader.read_exact(&mut data).await?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{duplex, AsyncWriteExt};

    #[tokio::test]
    async fn frames_roundtrip_with_strict_lengths() {
        let (mut a, mut b) = duplex(1024);
        let writer = tokio::spawn(async move {
            write_frame(&mut a, b"hello", 16).await.unwrap();
            write_frame(&mut a, b"world!", 16).await.unwrap();
        });
        assert_eq!(read_frame(&mut b, 16).await.unwrap(), b"hello");
        assert_eq!(read_frame(&mut b, 16).await.unwrap(), b"world!");
        writer.await.unwrap();
    }

    #[tokio::test]
    async fn oversized_length_is_rejected_before_reading_the_body() {
        let (mut a, mut b) = duplex(64);
        a.write_all(&100u32.to_be_bytes()).await.unwrap();
        a.write_all(b"x").await.unwrap();
        let error = read_frame(&mut b, 16).await.unwrap_err();
        assert!(matches!(error, Error::FrameTooLarge(100)));
    }

    #[tokio::test]
    async fn zero_length_and_oversized_writes_are_rejected() {
        let (mut a, b) = duplex(64);
        assert!(matches!(
            write_frame(&mut a, b"", 16).await.unwrap_err(),
            Error::Codec(_)
        ));
        assert!(matches!(
            write_frame(&mut a, &[0u8; 17], 16).await.unwrap_err(),
            Error::FrameTooLarge(17)
        ));
        drop(b);
    }
}
