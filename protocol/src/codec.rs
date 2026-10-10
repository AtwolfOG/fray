use anyhow::{ Result};
use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{constants::{HEADER_SIZE, MAX_PAYLOAD_SIZE, PROTOCOL_VERSION}, error::ProtocolError, frame::Frame};

#[derive(Debug)]
pub struct FrameWriter<W> {
  inner: W
}

impl<W: AsyncWrite + Unpin> FrameWriter<W> {
  pub fn new(inner: W) -> Self {
    Self { inner }
  }

  pub async fn write_frame(&mut self, frame: &Frame) -> Result<()>{ 
    let mut buf = BytesMut::with_capacity(HEADER_SIZE + frame.payload.len());
    frame.encode(&mut buf)?;
    self.inner.write_all(&buf).await?;
    Ok(())
  }
}

pub struct FrameReader<R> {
  inner: R,
  // buf: BytesMut,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
  pub fn new(inner: R) -> Self {
    Self { inner}
  }

  pub async fn read_frame(&mut self) -> Result<Frame> {
    let mut header = [0u8; HEADER_SIZE];
    self.inner.read_exact(&mut header).await?;
    if header[0] != PROTOCOL_VERSION {
      return Err(ProtocolError::InvalidVersion.into())
    }

    let len = u32::from_be_bytes([header[6], header[7], header[8], header[9]]);
    if len > MAX_PAYLOAD_SIZE as u32 {
      return Err(ProtocolError::PayloadTooLarge.into())
    }

    let mut payload_buffer = vec![0u8; len as usize];
    self.inner.read_exact(&mut payload_buffer).await?;

    let mut full_buffer = BytesMut::with_capacity(HEADER_SIZE + len as usize);
    full_buffer.extend_from_slice(&header);
    full_buffer.extend_from_slice(&payload_buffer);

    let frame = Frame::decode(full_buffer.as_ref())?;

    Ok(frame)
  }
  

}