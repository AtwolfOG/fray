use std::io::Cursor;

use bytes::{Buf, Bytes, BytesMut, BufMut};

use crate::{constants::{HEADER_SIZE, MAX_PAYLOAD_SIZE, PROTOCOL_VERSION}, error::ProtocolError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
  Auth = 1,
  AuthOK = 2,
  AuthErr = 3,
  RegisterTunnel = 4,
  TunnelRegistered = 5,
  TunnelErr = 6,
  OpenStream = 7,
  StreamOpened = 8,
  StreamClose = 9,
  Data = 10,
  Ping = 11,
  Pong = 12,
}

pub struct Frame {
  pub id: u32,
  pub kind: FrameType,
  pub payload: bytes::Bytes,
}

impl TryFrom<u8> for FrameType {
  type Error = ProtocolError;
  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      1 => Ok(FrameType::Auth),
      2 => Ok(FrameType::AuthOK),
      3 => Ok(FrameType::AuthErr),
      4 => Ok(FrameType::RegisterTunnel),
      5 => Ok(FrameType::TunnelRegistered),
      6 => Ok(FrameType::TunnelErr),
      7 => Ok(FrameType::OpenStream),
      8 => Ok(FrameType::StreamOpened),
      9 => Ok(FrameType::StreamClose),
      10 => Ok(FrameType::Data),
      11 => Ok(FrameType::Ping),
      12 => Ok(FrameType::Pong),
      _ => Err(ProtocolError::InvalidFrameType),
    }
  }
}

impl From<FrameType> for u8 {
  fn from(value: FrameType) -> Self {
    value as u8
  }
}


impl Frame {
  pub fn new(id: u32, kind: FrameType, payload: impl Into<Bytes>) -> Self {
    Self {
      id,
      kind,
      payload: payload.into(),
    }
  }

  pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError>{
    if buf.len() < HEADER_SIZE {
      return Err(ProtocolError::FrameTooSmall)
    }
    let mut cur = Cursor::new(buf);

    let version = cur.get_u8();
    if version != PROTOCOL_VERSION{
      return Err(ProtocolError::InvalidVersion)
    }

    let kind = FrameType::try_from(cur.get_u8())?;
    let id = cur.get_u32();
    let payload_len = cur.get_u32() as usize;

    if payload_len > MAX_PAYLOAD_SIZE{
      return Err(ProtocolError::PayloadTooLarge)
    }
    if cur.remaining() != payload_len{
      return Err(ProtocolError::InvalidFrame)
    }
    
    Ok(Frame::new(id, kind, Bytes::copy_from_slice(&buf[HEADER_SIZE..])))
  }

  pub fn encode(&self, out: &mut BytesMut) -> Result<(), ProtocolError>{
    if self.payload.len() > MAX_PAYLOAD_SIZE{
      return Err(ProtocolError::PayloadTooLarge)
    }
    
    out.put_u8(PROTOCOL_VERSION);
    out.put_u8(self.kind as u8);
    out.put_u32(self.id);
    out.put_u32(self.payload.len() as u32);
    out.put(self.payload.as_ref());
    Ok(())
  }
}