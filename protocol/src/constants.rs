pub const HEADER_SIZE: usize = 10;
pub const MAX_PAYLOAD_SIZE: usize = 1024 * 100; // 100KB
pub const MAX_FRAME_SIZE: usize = HEADER_SIZE + MAX_PAYLOAD_SIZE;
pub const PROTOCOL_VERSION: u8 = 1;
