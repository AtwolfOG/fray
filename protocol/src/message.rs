use bytes::{Buf, Bytes};

use crate::{error::ProtocolError, frame::{Frame, FrameType}};

#[derive(Debug)]
pub enum Message {
  Auth { id: i64, key: String },
  // AuthOK { user_id: u32 },
  AuthOK, 
  AuthErr { reason: String },
  RegisterTunnel,
  TunnelRegistered,
  // RegisterTunnel { host: Option<String>},
  // TunnelRegistered { host: String},
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
      Self::Auth { id, key } => (FrameType::Auth, [id.to_be_bytes().as_slice(), key.as_bytes()].concat()),
      Self::AuthOK => (FrameType::AuthOK, vec![]),
      Self::AuthErr { reason } => (FrameType::AuthErr, reason.into_bytes()),
      Self::RegisterTunnel => (FrameType::RegisterTunnel, vec![]),
      Self::TunnelRegistered => (FrameType::TunnelRegistered, vec![]),
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
      FrameType::Auth => Self::Auth { id: b.get_i64(), key: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::AuthOK => Self::AuthOK,
      FrameType::AuthErr => Self::AuthErr { reason: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::RegisterTunnel => Self::RegisterTunnel,
      FrameType::TunnelRegistered => Self::TunnelRegistered,
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