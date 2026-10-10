use std::{collections::HashMap, sync::{Arc, atomic::{AtomicU32, Ordering}}};

use thiserror::Error;
use crate::{client::handle_client, config::Config, public::handle_public};
use protocol::{codec::FrameWriter, message::Message};
use tokio::{io::{ WriteHalf}, net::{TcpListener, TcpStream}, sync::{Mutex, mpsc::{Sender}} };
use tracing::info;

#[derive(Debug, Clone)]
pub struct ServerState{
  senders: Arc<Mutex<HashMap<u32, Sender<Vec<u8>>>>>,
  client_frame_writers: Arc<Mutex<HashMap<u32, FrameWriter<WriteHalf<TcpStream>>>>>,
  next_public_id: Arc<AtomicU32>,
  next_tunnel_id: Arc<AtomicU32>,
}
pub struct Server{
  pub config: Config,
  state: ServerState,
}
#[derive(Debug, Error)]
pub enum ServerErr {
  #[error("Client already exists")]
  ClientAlreadyExists,
  #[error("Channel not found")]
  ChannelNotFound,
  // #[error("Channel closed")]
  // ChannelClosed,
  #[error("Failed to send message")]
  FailedToSend,
  #[error("Failed to generate id")]
  FailedToGenerateId,
}
impl Server{
  pub fn new(config: Config) -> Self {
    Self {
      config,
      state: ServerState {
        senders: Arc::new(Mutex::new(HashMap::new())),
        client_frame_writers: Arc::new(Mutex::new(HashMap::new())),
        // TODO: fix: 0 should be reserved for tunnel usage
        next_public_id: Arc::new(AtomicU32::new(1)),
        next_tunnel_id: Arc::new(AtomicU32::new(1)),
      },
    }
  }

  pub async fn run(&self){
    let client_addr = format!("{}:{}", self.config.addr, self.config.client_port);
    let public_addr = format!("{}:{}", self.config.addr, self.config.admin_port);

    let client_listener = TcpListener::bind(client_addr.clone()).await.expect("failed to bind");
    let public_listener = TcpListener::bind(public_addr.clone()).await.expect("failed to bind");
    info!("Listening on {}", client_addr);
    info!("Listening on {}", public_addr);

    let state = self.state.clone();
    
    let client_task = tokio::spawn(async move {
      loop {
        match client_listener.accept().await {
          Ok((client_socket, _)) => {
            let state = state.clone();
            tokio::spawn(async move {
              info!("accepted a client connection");
              match handle_client(client_socket, state).await {
                Ok(_) => info!("client connection closed"),
                Err(e) => tracing::error!("client connection error: {}", e),
              }
            });
          }
          Err(e) => {
            tracing::error!("Failed to accept client connection: {}", e);
          }
        }
      }
    });

    let state = self.state.clone();

    let public_task = tokio::spawn(async move {
      loop {
        match public_listener.accept().await {
          Ok((public_socket, _)) => {
            let state = state.clone();
            let (tx, rx) = tokio::sync::mpsc::channel(1024);
            tokio::spawn(async move {
              info!("accepted a public connection");
              match handle_public(public_socket, tx, rx, state).await {
                Ok(_) => info!("public connection closed"),
                Err(e) => tracing::error!("public connection error: {}", e),
              }
            });
          }
          Err(e) => {
            tracing::error!("Failed to accept public connection: {}", e);
          }
        }
      }
    });

    let _ = tokio::join!(client_task, public_task);
  }

}

impl ServerState {
  pub async fn add_sender(&self, client_id: u32, sender: Sender<Vec<u8>>) -> Result<(), ServerErr>{
    let mut idk = self.senders.lock().await;
    if idk.contains_key(&client_id){
      return Err(ServerErr::ClientAlreadyExists)
    }
    idk.insert(client_id, sender);
    Ok(())
  }
  
  pub async fn add_tunnel_frame_writer(&self, id: u32, frame_writer: FrameWriter<WriteHalf<TcpStream>>) -> Result<(), ServerErr>{
    let mut idk = self.client_frame_writers.lock().await;
    if idk.contains_key(&id){
      return Err(ServerErr::ClientAlreadyExists)
    }
    idk.insert(id, frame_writer);
    Ok(())
  }

  pub async fn send_data_to_channel(&self, id: u32, msg: Vec<u8>) -> Result<(), ServerErr>{
    let tx = self.senders.lock().await;
    match tx.get(&id) {
      Some(tx) => tx.send(msg).await.map_err(|_| ServerErr::FailedToSend),
      None => Err(ServerErr::ChannelNotFound),
    }
  }

  pub async fn write_message_to_tunnel(&self, id: u32, msg: Message) -> Result<(), ServerErr>{
    let mut fw = self.client_frame_writers.lock().await;
    match fw.get_mut(&id) {
      Some(fw) => fw.write_frame(&msg.into_frame(id)).await.map_err(|_| ServerErr::FailedToSend),
      None => Err(ServerErr::ChannelNotFound),
    }
  }


  pub async fn get_next_public_id(&self) -> Result<u32, ServerErr> {
    let id = self.next_public_id.fetch_add(1, Ordering::Relaxed);
    let mut tries = 0;
    loop {
      if tries > 10 {
        return Err(ServerErr::FailedToGenerateId);
      }
      if self.senders.lock().await.contains_key(&id){
        tries += 1;
        continue;
      }
      break;
    }
    Ok(id)
  }

  pub async fn get_next_tunnel_id(&self) -> Result<u32, ServerErr> {
    let id = self.next_tunnel_id.fetch_add(1, Ordering::Relaxed);
    let mut tries = 0;
    loop {
      if tries > 10 {
        return Err(ServerErr::FailedToGenerateId);
      }
      if self.client_frame_writers.lock().await.contains_key(&id){
        tries += 1;
        continue;
      }
      break;
    }
    Ok(id)
  }

  // this is for testing purpose
  pub async fn get_last_tunnel_id(&self) -> Result<u32, ServerErr> {
    let id = self.next_tunnel_id.load(Ordering::Relaxed);
    Ok(id - 1)
  }
}