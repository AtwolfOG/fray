use protocol::message::Message;
use tokio::{io::{AsyncWriteExt, AsyncReadExt, split}, net::TcpStream, select, sync::mpsc::Receiver};
use anyhow::{Context, Result};
use crate::client::ClientState;

pub async fn handle_request(stream: TcpStream, rx: &mut Receiver<Vec<u8>>, id: u32, client_state: ClientState) -> Result<()>{ 
  let ( mut reader, mut writer) = split(stream);
  let mut buffer = vec![0; 1024];

  
  loop {
    select! {
      rx_data = rx.recv() => {
        let data = rx_data.with_context(|| "nothing was sent to the channel")?;
        println!("sending data to public: {:?}", String::from_utf8_lossy(&data));
        match writer.write_all(&data).await {
          Ok(_) => {},
          Err(_) => break,
        };
      }

      data = reader.read(&mut buffer) => {
        
        match data {
          Ok(0) => break,
          Ok(data) => {
            println!("read data from local stream: {:?}", String::from_utf8_lossy(&buffer[..data]));
            let frame = Message::Data(buffer[..data].to_vec().into()).into_frame(id);
            // TODO: might handle it better
            client_state.send_frame_to_writer(frame).await?;
          }
          Err(_) => break,
        }
      }
    }
  }
  // send signal to close stream
  client_state.send_frame_to_writer(Message::StreamClose { stream_id: id }.into_frame(id)).await?;
  Ok(())
}