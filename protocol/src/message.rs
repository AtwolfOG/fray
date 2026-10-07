use bytes::{Buf, Bytes};

use crate::{error::ProtocolError, frame::{Frame, FrameType}};

pub enum Message {
  Auth { token: String },
  // AuthOK { user_id: u32 },
  AuthOK, 
  AuthErr { reason: String },
  RegisterTunnel { client_id: u32 },
  TunnelRegistered,
  TunnelErr { reason: String },
  OpenStream { tunnel_id: u32, stream_id: u32 },
  StreamOpened { tunnel_id: u32, stream_id: u32 },
  StreamClose,
  Data (Bytes),
  Ping,
  Pong,
}

impl Message{
  pub fn into_frame(self, id: u32) -> Frame{
    let (kind, payload) = match self {
      Self::Auth { token } => (FrameType::Auth, token.into_bytes()),
      Self::AuthOK => (FrameType::AuthOK, vec![]),
      Self::AuthErr { reason } => (FrameType::AuthErr, reason.into_bytes()),
      Self::RegisterTunnel { client_id } => (FrameType::RegisterTunnel, client_id.to_be_bytes().to_vec()),
      Self::TunnelRegistered => (FrameType::TunnelRegistered, vec![]),
      Self::TunnelErr { reason } => (FrameType::TunnelErr, reason.into_bytes()),
      Self::OpenStream { tunnel_id, stream_id } => (FrameType::OpenStream, [tunnel_id.to_be_bytes(), stream_id.to_be_bytes()].concat()),
      Self::StreamOpened { tunnel_id, stream_id } => (FrameType::StreamOpened, [tunnel_id.to_be_bytes(), stream_id.to_be_bytes()].concat()),
      Self::StreamClose => (FrameType::StreamClose, vec![]),
      Self::Data (payload) => (FrameType::Data, payload.to_vec()),
      Self::Ping => (FrameType::Ping, vec![]),
      Self::Pong => (FrameType::Pong, vec![]),
    };
    Frame::new(id, kind, payload)
  }

  pub fn from_frame(frame: Frame) -> Result<Self, ProtocolError>{
    let mut b = frame.payload;
    Ok(match frame.kind{
      FrameType::Auth => Self::Auth { token: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::AuthOK => Self::AuthOK,
      FrameType::AuthErr => Self::AuthErr { reason: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::RegisterTunnel => Self::RegisterTunnel { client_id: b.get_u32() },
      FrameType::TunnelRegistered => Self::TunnelRegistered,
      FrameType::TunnelErr => Self::TunnelErr { reason: String::from_utf8(b.to_vec()).map_err(|_| ProtocolError::MalformedPayload)? },
      FrameType::OpenStream => Self::OpenStream { tunnel_id: b.get_u32(), stream_id: b.get_u32() },
      FrameType::StreamOpened => Self::StreamOpened { tunnel_id: b.get_u32(), stream_id: b.get_u32() },
      FrameType::StreamClose => Self::StreamClose,
      FrameType::Data => Self::Data(b),
      FrameType::Ping => Self::Ping,
      FrameType::Pong => Self::Pong,
    })
  }
}