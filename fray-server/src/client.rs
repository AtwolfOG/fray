use std::time::Duration;

use protocol::{frame::FrameType, message::Message};
use tokio::{net::TcpStream, sync::mpsc::{Receiver, Sender}, time::Instant};
use anyhow::{Context, Result, bail};
use tokio::io::{BufReader, split};
use tracing::info;
use crate::{server::ServerState, utils::generate_host};
use protocol::codec::{FrameReader, FrameWriter};

pub async fn handle_client(stream: TcpStream, server_state: ServerState) -> Result<()>{
  
  let (reader, writer) = split(stream);
  let reader = BufReader::new(reader);



  let mut frame_reader = FrameReader::new(reader);
  let mut frame_writer = FrameWriter::new(writer);

  // authenticate client
  info!("authenticating client");
  let auth_frame = frame_reader.read_frame().await?;
  let Message::Auth {id: client_id, token} = Message::from_frame(auth_frame)? else {
    // auth failed
    // write authErr to client
    frame_writer.write_frame(&Message::AuthErr { reason: "authentication error".to_owned() }.into_frame(0)).await.with_context(|| "Unable to write error frame after auth failure")?;
    bail!("Invalid auth message")
  };

  let Ok(client) = server_state.auth_client(client_id, &token).await else {
    // auth failed
    // write authErr to client
    frame_writer.write_frame(&Message::AuthErr { reason: "authentication error".to_owned() }.into_frame(0)).await.with_context(|| format!("Unable to write error frame after auth failure for client id: {client_id}"))?;
    bail!("Invalid client credentials")
  };

  info!("authenticated client: {}, id: {}", client.name, client.id);
  let auth_ok_frame = Message::AuthOK.into_frame(0);
  frame_writer.write_frame(&auth_ok_frame).await.with_context(|| format!("Unable to write auth ok frame for client id: {client_id}"))?;

  // get host
  let host = generate_host();

  // register tunnel
  let register_tunnel_frame = frame_reader.read_frame().await?;
  let Message::RegisterTunnel = Message::from_frame(register_tunnel_frame)? else {
    // invalid register tunnel frame
    // write tunnelErr to client
    frame_writer.write_frame(&Message::TunnelErr { reason: "invalid register tunnel frame".to_owned() }.into_frame(0)).await.with_context(|| format!("Unable to write error frame after register tunnel failure for client id: {client_id}"))?;
    bail!("Invalid register tunnel message")
  };

  // save frame_writer to server_state
  server_state.add_client(host.clone(), frame_writer, client_id).await;

  // write tunnel registered to client
  server_state.write_message_to_tunnel(&host, 0, Message::TunnelRegistered).await.with_context(|| format!("Unable to write tunnel registered frame for client id: {client_id}"))?;

  let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

  // spawn ping task
  tokio::spawn(ping_client(server_state.clone(), host.clone()));

  // spawn close client with timeout
  tokio::spawn(close_client_with_timeout(rx, server_state.clone(), host.clone()));
  
  loop {
    let frame = frame_reader.read_frame().await?;
    match frame.kind {
      FrameType::Data => {
        let id = frame.id;
        let data = frame.payload.to_vec();
        if data.is_empty() {
          continue;
        }
        server_state.send_data_to_channel(id, data).await?;
      }
      FrameType::TunnelErr => {
        todo!()
      }
      FrameType::CloseStream => {
        let message = Message::from_frame(frame)?;
        let Message::CloseStream { stream_id: session_id } = message else {
          continue;
        };
        server_state.remove_sender(session_id).await?;

        // write stream closed to client
        let _ = server_state.write_message_to_tunnel(&host, session_id, Message::StreamClosed { stream_id: session_id }).await;
      }
      FrameType::Pong => {
        // handle pong
        let _ = handle_pong(tx.clone()).await;
      }
      _ => {
        tracing::error!("invalid frame type: {:?}", frame.kind);
        let msg = Message::from_frame(frame);
        tracing::error!("invalid message: {:?}", msg);
      }
    }
  }
}

async fn handle_pong(tx: Sender<()>)-> Result<()>{
  tx.send(()).await.map_err(|_| anyhow::anyhow!("Failed to send pong"))?;
  Ok(())
}

async fn close_client_with_timeout(mut rx: Receiver<()>, server_state: ServerState, host: String){
  let ping_timer = tokio::time::sleep(Duration::from_secs(60));

  tokio::pin!(ping_timer);
  
  loop {
    tokio::select!{
      () = &mut ping_timer => {
        server_state.remove_client(&host).await;        
        tracing::info!("client timed out: {}", host);
        break;
      }
      _ = rx.recv() => {
        let new_deadline = Instant::now() + Duration::from_secs(60);
        ping_timer.as_mut().reset(new_deadline);
        continue;
      }
    }
  } 
}

async fn ping_client(server_state: ServerState, host: String) -> Result<()> { 
  let mut interval = tokio::time::interval(Duration::from_secs(45));
  loop {
    interval.tick().await;
    let ping_msg = Message::Ping;
    server_state.write_message_to_tunnel(&host, 0, ping_msg).await.with_context(|| "Unable to write ping frame to client")?;
  }
}