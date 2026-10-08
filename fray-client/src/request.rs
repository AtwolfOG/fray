use protocol::message::Message;
use tokio::{io::{AsyncWriteExt, AsyncReadExt, split}, net::TcpStream, select, sync::mpsc::Receiver};
use anyhow::Result;
use crate::client::ClientState;

pub async fn handle_request(stream: TcpStream, rx: &mut Receiver<Vec<u8>>, id: u32, client_state: ClientState) -> Result<()>{ 
  let ( mut reader, mut writer) = split(stream);
  let mut buffer = Vec::new();

  
  loop {
    select! {
      rx_data = rx.recv() => {
        let data = rx_data.unwrap();
        match writer.write_all(&data).await {
          Ok(_) => {},
          Err(_) => break,
        };
      }

      data = reader.read(&mut buffer) => {
        match data {
          Ok(0) => break,
          Ok(data) => {
            let frame = Message::Data(buffer[..data].to_vec().into()).into_frame(id);
            // TODO: might handle it better
            let _ = client_state.send_frame_to_writer(frame).await;
          }
          Err(_) => break,
        }
      }
    }
  }
  Ok(())
}