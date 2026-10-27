use std::{collections::HashMap, sync::Arc};

use anyhow::{Context, Result};
use protocol::codec::{FrameReader, FrameWriter};
use protocol::frame::{Frame, FrameType};
use protocol::message::Message;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::{io::split};
use tokio::sync::mpsc::{channel, Sender};
use tracing::{error, info};

use crate::cli::RunArgs;
use crate::request::handle_request;

#[derive(Debug, Clone)]
pub struct ClientState {
  senders: Arc<Mutex<HashMap<u32, Sender<Vec<u8>>>>>,
  client_frame_writer: Option<Arc<Mutex< FrameWriter<WriteHalf<TcpStream>>>>>, 
}

pub struct Client {
  state: ClientState,
  config: RunArgs,
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
  pub fn new(config: RunArgs) -> Self {
    Self {
      state: ClientState {
        senders: Arc::new(Mutex::new(HashMap::new())),
        client_frame_writer: None,
      },
      config,
    }
  }

  pub async fn run(&mut self) -> Result<()> {
    let stream = TcpStream::connect(&self.config.remote_addr).await.map_err(|_| ClientErr::ConnectError).with_context(|| format!("Failed to connect to server: {}", self.config.remote_addr))?;
    info!("Connected to server: {}", self.config.remote_addr);
    // split stream
    let (reader, writer) = split(stream);
    let writer = FrameWriter::new(writer);
    let mut reader = FrameReader::new(reader);
    self.state.client_frame_writer = Some(Arc::new(Mutex::new(writer)));

    // send auth
    info!("sending auth request");
    let auth_frame = Message::Auth { key: self.config.key.clone() }.into_frame(0);
    self.state.send_frame_to_writer(auth_frame).await?;

    // handle auth response
    info!("waiting for auth response");
    let auth_res_frame = reader.read_frame().await.with_context(|| "Failed to read auth response")?;
    match Message::from_frame(auth_res_frame) {
      Ok(Message::AuthOK) => {
        info!("Authentication successful");
      }
      Ok(Message::AuthErr { reason }) => {
        return Err(anyhow::anyhow!("Authentication failed: {}", reason));
      }
      _ => {
        return Err(anyhow::anyhow!("Invalid auth response"));
      }
    };

    // register tunnel
    info!("registering tunnel");
    let register_tunnel_frame = Message::RegisterTunnel.into_frame(0);
    self.state.send_frame_to_writer(register_tunnel_frame).await?;

    // handle tunnel registered response
    info!("waiting for tunnel registered response");
    let tunnel_registered_frame = reader.read_frame().await.with_context(|| "Failed to read tunnel registered response")?;
    match Message::from_frame(tunnel_registered_frame) {
      Ok(Message::TunnelRegistered) => {
        info!("Tunnel registered successfully");
      }
      Ok(Message::TunnelErr { reason }) => {
        return Err(anyhow::anyhow!("Tunnel registration failed: {}", reason));
      }
      _ => {
        return Err(anyhow::anyhow!("Invalid tunnel registered response"));
      }
    };
    
    loop {
      // println!("trying to read frame");
      let frame = reader.read_frame().await?;
      // println!("frame read: {:?}", frame);
      match frame.kind {
        FrameType::OpenStream => {
          let id = frame.id;
          // if sender already exists, skip this frame
          if self.state.check_if_sender_exists(id).await {
            continue;
          }
          // connect to the local server using tcp
          // let addr = format!("{}:{}", "host.docker.internal", self.config.local_port);
          let addr = format!("{}:{}", "localhost", self.config.local_port);
          // TODO: handle error when local server is not reachable
          let stream = TcpStream::connect(addr).await?;
          let (tx, mut rx) = channel(100);
          match self.state.add_sender(id, tx).await {
            Ok(_) => {
              let client_state = self.state.clone();
              // spawn a new task to handle the request
              tokio::spawn(async move {
                if let Err(e) = handle_request(stream, &mut rx, id, client_state.clone()).await {
                  error!("Failed to handle request for id {}: {}", id, e);
                }
                // remove sender from hashmap after request is done
                if let Err(e) = client_state.remove_sender(id).await {
                  error!("Failed to remove sender for id {}: {}", id, e);
                };
              });
            },
            // TODO: handle error
            Err(_) => continue,
          }
        }
        FrameType::Data => {
          let id = frame.id;
          println!("data frame read with id: {}", id);
          if self.state.check_if_sender_exists(id).await {
            let Ok(_) = self.state.send_data_to_channel(id, frame.payload.to_vec()).await else {
              error!("Failed to send data to channel for id: {}", id);
              continue;
            };
          };
          
        }
        FrameType::Ping => {
          self.state.send_frame_to_writer(Message::Pong.into_frame(0)).await?;
        }
        _ => {
          error!("Unknown frame type: {:?}", frame.kind);
        }
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