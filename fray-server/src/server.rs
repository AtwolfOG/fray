use std::{collections::HashMap, sync::{Arc, atomic::{AtomicU32, Ordering}}};

use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use thiserror::Error;
use crate::{cli::RunArgs, client::handle_client, public::handle_public};
use protocol::{codec::FrameWriter, message::Message};
use tokio::{io::{ WriteHalf}, net::{TcpListener, TcpStream}, sync::{Mutex, mpsc::{Sender}} };
use tracing::info;

#[derive(Debug, Clone)]
pub struct ServerState{
  senders: Arc<Mutex<HashMap<u32, Sender<Vec<u8>>>>>,
  client_frame_writers: Arc<Mutex<HashMap<String, FrameWriter<WriteHalf<TcpStream>>>>>,
  host_to_client_id: Arc<Mutex<HashMap<String, i64>>>,
  next_session_id: Arc<AtomicU32>,
  pub db: DatabaseConnection,
}
pub struct Server{
  pub config: RunArgs,
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
  #[error("Failed to get client")]
  FailedToGetClient,
  // #[error("Failed to save req")]
  // FailedToSaveReq,
  // #[error("Failed to save resp")]
  // FailedToSaveResp,
}
impl Server{
  pub fn new(config: RunArgs, db: DatabaseConnection) -> Self {
    Self {
      config,
      state: ServerState::build(db),
    }
  }

  pub async fn run(&self) -> Result<()>{
    let client_addr = format!("{}:{}", self.config.addr, self.config.client_port);
    let public_addr = format!("{}:{}", self.config.addr, self.config.public_port);

    let client_listener = TcpListener::bind(client_addr.clone()).await.with_context(|| format!("Failed to bind to client port: {}", client_addr))?;
    let public_listener = TcpListener::bind(public_addr.clone()).await.with_context(|| format!("Failed to bind to public port: {}", public_addr))?;
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
            let (tx, rx) = tokio::sync::mpsc::channel(100);
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
    Ok(())
  }

}

impl ServerState {

  pub fn build(db: DatabaseConnection) -> Self{
   Self {
      senders: Arc::new(Mutex::new(HashMap::new())),
      client_frame_writers: Arc::new(Mutex::new(HashMap::new())),
      host_to_client_id: Arc::new(Mutex::new(HashMap::new())),
      next_session_id: Arc::new(AtomicU32::new(1)),
      db,
    }
  }
  /// add public sender using session_id
  pub async fn add_sender(&self, session_id: u32, sender: Sender<Vec<u8>>) -> Result<(), ServerErr>{
    let mut idk = self.senders.lock().await;
    if idk.contains_key(&session_id){
      return Err(ServerErr::ClientAlreadyExists)
    }
    idk.insert(session_id, sender);
    Ok(())
  }

  /// send data to channel using session_id
  pub async fn send_data_to_channel(&self, session_id: u32, msg: Vec<u8>) -> Result<(), ServerErr>{
    let tx = self.senders.lock().await;
    match tx.get(&session_id) {
      Some(tx) => tx.send(msg).await.map_err(|_| ServerErr::FailedToSend),
      None => Err(ServerErr::ChannelNotFound),
    }
  }

  /// remove sender using session_id
  pub async fn remove_sender(&self, session_id: u32) -> Result<(), ServerErr>{
    let mut idk = self.senders.lock().await;
    idk.remove(&session_id);
    Ok(())
  }
  
  /// add tunnel frame writer using host
  async fn add_tunnel_frame_writer(&self, host: String, frame_writer: FrameWriter<WriteHalf<TcpStream>>) {
    let mut idk = self.client_frame_writers.lock().await;
    // todo: handle reconnection better instead of just overriding
    // override if the client reconnects
    idk.insert(host, frame_writer);
  }
  
  /// remove tunnel frame writer using host
  async fn remove_tunnel_frame_writer(&self, host: &String){
    let mut idk = self.client_frame_writers.lock().await;
    idk.remove(host);
  }

  /// write message to tunnel using host and request id
  pub async fn write_message_to_tunnel(&self, host: &String, req_id: u32, msg: Message) -> Result<(), ServerErr>{
    let mut fw = self.client_frame_writers.lock().await;
    match fw.get_mut(host) {
      Some(fw) => fw.write_frame(&msg.into_frame(req_id)).await.map_err(|_| ServerErr::FailedToSend),
      None => Err(ServerErr::ChannelNotFound),
    }
  }

  /// get client id from host
  pub async fn get_client_id_from_host(&self, host: String) -> Result<i64, ServerErr> {
    let idk = self.host_to_client_id.lock().await;
    match idk.get(&host) {
      Some(id) => Ok(*id),
      None => Err(ServerErr::ChannelNotFound),
    }
  }

  /// remove host from client id map
  async fn remove_host_from_client_id_map(&self, host: &String) {
    let mut idk = self.host_to_client_id.lock().await;
    idk.remove(host);
  }

  /// add host to client id map
  async fn add_host_to_client_id(&self, host: String, client_id: i64) {
    let mut idk = self.host_to_client_id.lock().await;
    idk.insert(host, client_id);
  }

  /// this remove stream writers from tunnel_writers, and remove host from host_to_client_id map
  pub async fn remove_client(&self, host: &String) {
    self.remove_tunnel_frame_writer(host).await;
    self.remove_host_from_client_id_map(host).await;
  }

  /// this add tunnel frame writer to tunnel_writers, and add host to host_to_client_id map
  pub async fn add_client(&self, host: String, frame_writer: FrameWriter<WriteHalf<TcpStream>>, client_id: i64) {
    self.add_tunnel_frame_writer(host.clone(), frame_writer).await;
    self.add_host_to_client_id(host, client_id).await;
  }

  /// get next public id
  pub async fn get_next_session_id(&self) -> Result<u32, ServerErr> {
    let id = self.next_session_id.fetch_add(1, Ordering::Relaxed);
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

  /// get client using key
  pub async fn get_client_with_key(&self, key: &str) -> Result<crate::entity::client::Model, ServerErr> {
    let client = crate::entity::client::Entity::find().filter(crate::entity::client::Column::Key.eq(key)).one(&self.db).await.map_err(|_| ServerErr::FailedToGetClient)?;
    match client {
      Some(client) => Ok(client),
      None => Err(ServerErr::FailedToGetClient),
    }
  }

}