use std::io::Cursor;

use anyhow::{Result};
use protocol::message::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::server::ServerState;
use crate::utils;

pub async fn handle_public(stream: TcpStream, tx: Sender<Vec<u8>>, mut rx: Receiver<Vec<u8>>, server_state: ServerState) -> Result<()> {
  let (reader, mut writer) = split(stream);
  let mut reader = BufReader::new(reader);
  let mut buf = vec![0u8; 1024];
  let mut  read_open = true;
  let id = server_state.get_next_public_id().await?;
  server_state.add_sender(id, tx).await?;


  // read headers
  let (headers, header_buf) = utils::get_headers(&mut reader).await?;
  let host = headers.get("host").ok_or(anyhow::anyhow!("Host header not found"))?;
  let host = host.split('.').next().ok_or(anyhow::anyhow!("Invalid host header"))?;

  let Ok(tunnel_id) = server_state.get_host_to_tunnel(host).await else {
    tracing::error!("Host {} not found", host);
    return Err(anyhow::anyhow!("Host {} not found", host));
  };
  
  let mut reader = Cursor::new(header_buf).chain(reader);

  loop {
    tokio::select! {

      n = reader.read(&mut buf), if read_open => {
        let n = n?;
        if n == 0{
          read_open = false;
        }
        let data = buf[..n].to_vec();
        let msg = Message::Data(data.into());
        server_state.write_message_to_tunnel(tunnel_id, msg).await?;
      }
      
      data = rx.recv() => {
        match data {
          Some(data) => {
            writer.write_all(&data).await?;
          }
          None => break,
        }
      }
    }
  }
  
  Ok(())
}
