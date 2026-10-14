use migration::MigratorTrait;
use protocol::frame::FrameType;
use tokio::net::TcpStream;
use anyhow::Result;
use tokio::io::{BufReader, split};
use crate::{server::ServerState, utils};
use protocol::codec::{FrameReader, FrameWriter};

pub async fn handle_client(stream: TcpStream, server_state: ServerState) -> Result<()>{
  
  let (reader, writer) = split(stream);
  let reader = BufReader::new(reader);


  // TODO: Implement proper authentication
  let id = server_state.get_next_tunnel_id().await?;

  let mut frame_reader = FrameReader::new(reader);
  let frame_writer = FrameWriter::new(writer);

  // save frame_writer to server_state
  server_state.add_tunnel_frame_writer(id, frame_writer).await?;
  // get and map host to tunnel id
  let host = utils::generate_host();
  server_state.set_host_to_tunnel(host, id).await;
  
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
      _ => {
        // TODO: Handle other frame types
      }
    }
  }
}