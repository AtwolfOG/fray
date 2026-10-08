use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use protocol::codec::{FrameReader, FrameWriter};
use protocol::frame::{Frame, FrameType};
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::{io::split};
use tokio::sync::mpsc::{channel, Sender};
use tracing::{error, info};

use crate::config::Config;
use crate::request::handle_request;

#[derive(Debug, Clone)]
pub struct ClientState {
  senders: Arc<Mutex<HashMap<u32, Sender<Vec<u8>>>>>,
  client_frame_writer: Option<Arc<Mutex< FrameWriter<WriteHalf<TcpStream>>>>>, 
}

pub struct Client {
  state: ClientState,
  config: Config,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientErr {
  #[error("Failed to bind to address")]
  ConnectError,
  // #[error("Failed to send data to channel")]
  // SenderError,
  #[error("Sender already exists for id {}", _0)]
  SenderExists(u32),
}

impl Client {
  pub fn new(config: Config) -> Self {
    Self {
      state: ClientState {
        senders: Arc::new(Mutex::new(HashMap::new())),
        client_frame_writer: None,
      },
      config,
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    let stream = TcpStream::connect(&self.config.remote_addr).await.map_err(|_| ClientErr::ConnectError)?;
    info!("Connected to server: {}", self.config.remote_addr);
    let (reader, writer) = split(stream);
    let writer = FrameWriter::new(writer);
    let mut reader = FrameReader::new(reader);
    self.state.client_frame_writer = Some(Arc::new(Mutex::new(writer)));
    
    loop {
      let frame = reader.read_frame().await?;
      match frame.kind {
        FrameType::Data => {
          let id = frame.id;
          if self.state.check_if_sender_exists(id).await {
            let _ = self.state.send_data_to_channel(id, frame.payload.to_vec()).await;
            continue;
          }
          // connect to the local server using tcp
          let addr = format!("{}:{}", "0.0.0.0", self.config.listen_port);
          let stream = TcpStream::connect(addr).await.map_err(|_| ClientErr::ConnectError)?;
          let (tx, mut rx) = channel(100);
          match self.state.add_sender(id, tx).await {
            Ok(_) => {
              let client_state = self.state.clone();
              tokio::spawn(async move {
                if let Err(e) = handle_request(stream, &mut rx, id, client_state.clone()).await {
                  error!("Failed to handle request for id {}: {}", id, e);
                  let _ = client_state.remove_sender(id).await;
                }
              });
            },
            Err(ClientErr::SenderExists(id)) => {
              info!("Sender already exists for id {}", id);
              let payload = frame.payload.to_vec();
              let _ = self.state.send_data_to_channel(id, payload).await;
              continue;
            },
            _ => continue,
          }
          
        }
        FrameType::Auth => {},
        _ => {}
      }
    }
  }
}

impl ClientState {
  pub async fn add_sender(&self, id: u32, sender: Sender<Vec<u8>>) -> Result<(), ClientErr>{
    let mut senders = self.senders.lock().await;
    if senders.contains_key(&id) {
      return Err(ClientErr::SenderExists(id));
    }
    senders.insert(id, sender);
    Ok(())
  }
  
  pub async fn check_if_sender_exists(&self, id: u32) -> bool {
    let senders = self.senders.lock().await;
    senders.contains_key(&id)
  }

  pub async fn remove_sender(&self, id: u32) -> Result<()>{
    let mut senders = self.senders.lock().await;
    senders.remove(&id);
    Ok(())
  }

  pub async fn send_data_to_channel(&self, id: u32, data: Vec<u8>) -> Result<()> {
    let txs = self.senders.lock().await;
    match txs.get(&id) {
      Some(tx) => tx.send(data).await.map_err(|_| anyhow::anyhow!("Failed to send data to channel for id {}", id)),
      None => Err(anyhow::anyhow!("Sender not found for id {}", id)),
    }
  }

  pub async fn send_frame_to_writer(&self, frame: Frame) -> Result<()> {
    match &self.client_frame_writer{
      Some(writer) => writer.lock().await.write_frame(&frame).await.map_err(|_| anyhow::anyhow!("Failed to send frame to writer")),
      None => Err(anyhow::anyhow!("Writer not found")),
    }
  }
}