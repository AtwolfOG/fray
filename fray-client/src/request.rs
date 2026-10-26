use protocol::message::Message;
use tokio::{io::{AsyncWriteExt, AsyncReadExt, split}, net::TcpStream, select, sync::mpsc::Receiver};
use anyhow::{Context, Result};
use crate::client::ClientState;

pub async fn handle_request(stream: TcpStream, rx: &mut Receiver<Vec<u8>>, id: u32, client_state: ClientState) -> Result<()>{
  let ( mut reader, mut writer) = split(stream);
  let mut buffer = vec![0; 1024];

  // let mut request_payload = Vec::<u8>::new();
  // let mut response_payload = Vec::<u8>::new();
  
  loop {
    select! {
      rx_data = rx.recv() => {
        let data = rx_data.with_context(|| "nothing was sent to the channel")?;
        // request_payload.extend(&data);
        match writer.write_all(&data).await {
          Ok(_) => {},
          Err(_) => break,
        };
            println!("data read from remote stream: {:?}", String::from_utf8_lossy(&data));
      }

      n = reader.read(&mut buffer) => {
        
        match n {
          Ok(0) => break,
          Ok(n) => {
            // response_payload.extend(&buffer[..n]);
            println!("read data from local stream: {:?}", String::from_utf8_lossy(&buffer[..n]));
            let frame = Message::Data(buffer[..n].to_vec().into()).into_frame(id);
            // TODO: might handle it better
            client_state.send_frame_to_writer(frame).await?;
          }
          Err(_) => break,
        }
      }
    }
  }
  // send signal to close stream
  client_state.send_frame_to_writer(Message::CloseStream { stream_id: id }.into_frame(id)).await?;
  Ok(())
}