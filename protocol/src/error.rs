use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
  #[error("Io Error: {0}")]
  Io(#[from] std::io::Error),

  #[error("frame too small")]
  FrameTooSmall,

  #[error("Invalid frame type")]
  InvalidFrameType,
  
  #[error("Invalid frame")]
  InvalidFrame,

  #[error("Invalid protocol version")]
  InvalidVersion,

  #[error("Malformed payload")]
  MalformedPayload,

  #[error("Payload too large")]
  PayloadTooLarge,

  #[error("Authentication failed")]
  AuthenticationFailed,

  #[error("Tunnel not found")]
  TunnelNotFound,

  #[error("Stream not found")]
  StreamNotFound,

  #[error("Internal error: {0}")]
  Internal(String),
}