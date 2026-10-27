use bytes::{Buf, Bytes};

use crate::{error::ProtocolError, frame::{Frame, FrameType}};

#[derive(Debug)]
pub enum Message {
  Auth { key: String },
  // AuthOK { user_id: u32 },
  AuthOK, 
  AuthErr { reason: String },
  RegisterTunnel { host: Option<String> },
  TunnelRegistered { host: String },
  TunnelErr { reason: String },
  OpenStream { stream_id: u32 },
  StreamOpened { stream_id: u32 },
  CloseStream { stream_id: u32 },
  StreamClosed { stream_id: u32 },
  Data (Bytes),
  Ping,
  Pong,
}

impl Message{
  pub fn into_frame(self, id: u32) -> Frame{
    let (kind, payload) = match self {
      Self::Auth { key } => (FrameType::Auth, key.into_bytes()),
      Self::AuthOK => (FrameType::AuthOK, vec![]),
      Self::AuthErr { reason } => (FrameType::AuthErr, reason.into_bytes()),
      Self::RegisterTunnel { host } => (FrameType::RegisterTunnel, host.map(|h| h.into_bytes()).unwrap_or_else(|| vec![])), 
      Self::TunnelRegistered { host } => (FrameType::TunnelRegistered, host.into_bytes()),
      Self::TunnelErr { reason } => (FrameType::TunnelErr, reason.into_bytes()),
      Self::OpenStream { stream_id } => (FrameType::OpenStream, [stream_id.to_be_bytes().as_slice()].concat()),
      Self::StreamOpened { stream_id } => (FrameType::StreamOpened, [stream_id.to_be_bytes().as_slice()].concat()),
      Self::CloseStream { stream_id } => (FrameType::CloseStream, stream_id.to_be_bytes().to_vec()),
      Self::StreamClosed { stream_id } => (FrameType::StreamClosed, stream_id.to_be_bytes().to_vec()),
      Self::Data (payload) => (FrameType::Data, payload.to_vec()),
      Self::Ping => (FrameType::Ping, vec![]),
      Self::Pong => (FrameType::Pong, vec![]),
    };
    Frame::new(id, kind, payload)
  }

  pub fn from_frame(frame: Frame) -> Result<Self, ProtocolError>{
    let mut b = frame.payload;
    Ok(match frame.kind{
      FrameType::Auth => Self::Auth { key: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::AuthOK => Self::AuthOK,
      FrameType::AuthErr => Self::AuthErr { reason: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::RegisterTunnel => Self::RegisterTunnel { host: if b.is_empty() { None } else { Some(String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)?) } },
      FrameType::TunnelRegistered => Self::TunnelRegistered { host: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::TunnelErr => Self::TunnelErr { reason: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::OpenStream => Self::OpenStream { stream_id: b.get_u32() },
      FrameType::StreamOpened => Self::StreamOpened { stream_id: b.get_u32() },
      FrameType::CloseStream => Self::CloseStream { stream_id: b.get_u32() },
      FrameType::StreamClosed => Self::StreamClosed { stream_id: b.get_u32() },
      FrameType::Data => Self::Data(b),
      FrameType::Ping => Self::Ping,
      FrameType::Pong => Self::Pong,
    })
  }
}