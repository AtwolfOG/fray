use anyhow::{Ok, Result};
use protocol::message::Message;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::server::ServerState;

pub async fn handle_public(stream: TcpStream, tx: Sender<Vec<u8>>, mut rx: Receiver<Vec<u8>>, server_state: ServerState) -> Result<()> {
  let (reader, mut writer) = split(stream);
  let mut reader = BufReader::new(reader);
  let mut buf = vec![0u8; 4096];
  let mut  read_open = true;
  let id = server_state.get_next_public_id().await?;
  server_state.add_sender(id, tx).await?;
  

  loop {
    tokio::select! {

      data = reader.read(&mut buf), if read_open => {
        let n = data?;
        if n > 0{
          read_open = false;
        }
        let msg = Message::Data(buf.to_vec().into());
        server_state.write_message_to_tunnel(id, msg).await?;
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
