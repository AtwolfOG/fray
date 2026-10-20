use std::io::Cursor;

use anyhow::{Context, Result};
use protocol::message::Message;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, info};

use crate::server::ServerState;
use crate::{entity, utils};

pub async fn handle_public(stream: TcpStream, tx: Sender<Vec<u8>>, mut rx: Receiver<Vec<u8>>, server_state: ServerState) -> Result<()> {
  let (reader, mut writer) = split(stream);
  let mut reader = BufReader::new(reader);
  let mut buf = vec![0u8; 1024];
  let mut  read_open = true;
  let session_id = server_state.get_next_session_id().await?;
  server_state.add_sender(session_id, tx).await?;


  // read headers
  info!("api endpoint hit.");
  let (headers, header_buf) = utils::get_headers(&mut reader).await?;
  let host = headers.get("host").ok_or(anyhow::anyhow!("Host header not found"))?;
  let host = if host.len() < 3 {
    return Err(anyhow::anyhow!("Invalid host header"));
  } else {
    host.split('.').next().ok_or(anyhow::anyhow!("Invalid host header: no subdomain"))?.to_owned()
  };

  let client_id = server_state.get_client_id_from_host(host.clone()).await?;

  let mut reader = Cursor::new(header_buf).chain(reader);

  // for saving to db
  let mut request_payload = Vec::<u8>::new();
  let mut response_payload = Vec::<u8>::new();
  let start = std::time::Instant::now();

  let msg = Message::OpenStream { stream_id: session_id };
  server_state.write_message_to_tunnel(&host, session_id, msg).await.with_context(|| format!("failed to open stream on client: {}", client_id))?;

  loop {
    tokio::select! {

      n = reader.read(&mut buf), if read_open => {
        let n = n?;
        if n == 0{
          read_open = false;
          continue;
          
        }
        let data = buf[..n].to_vec();
        request_payload.extend_from_slice(&buf[..n]);
        let msg = Message::Data(data.into());
        server_state.write_message_to_tunnel(&host, session_id, msg).await?;
      }
      
      data = rx.recv() => {
        match data {
          Some(data) => {
            response_payload.extend_from_slice(&data);
            writer.write_all(&data).await?;
          }
          None => break,
        }
      }
    }
  }
  let duration = start.elapsed().as_millis() as i64;

  // determine status according to the len of the response
  let status = if response_payload.is_empty() {
    "failure".to_owned()
  } else {
    "success".to_owned()
  };

  let api_log = entity::api_log::ActiveModel {
    client_id: Set(client_id),
    request_payload: Set(request_payload),
    response_payload: Set(response_payload),
    duration_ms: Set(duration),
    status: Set(status),
    ..Default::default()
  };
  // log db err
  if let Err(e) = api_log.insert(&server_state.db).await { 
    error!("Failed to insert api log: {}", e) 
  };

  Ok(())
}
