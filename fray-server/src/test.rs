use std::time::Duration;

use protocol::codec::{FrameReader, FrameWriter};
use protocol::frame::FrameType;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

use crate::public::handle_public;
use crate::server::ServerState;
use crate::{db, entity};

pub async fn setup_and_seed_db() -> DatabaseConnection {
  let db = db::setup_database(Some("sqlite::memory:?cache=shared")).await.expect("Failed to connect to database");
  let now = chrono::Utc::now().into();
  let client = entity::client::ActiveModel {
    name: Set("test_client".to_string()),
    key: Set("test_secret".to_string()),
    created_at: Set(now),
    updated_at: Set(now),
    ..Default::default()
  };
  let _ = client.insert(&db).await;
  db
}

#[tokio::test]
async fn test_handle_public_request_bytes_not_duplicated() {
  let db = setup_and_seed_db().await;
  let client = entity::client::Entity::find()
    .filter(entity::client::Column::Host.eq("fray"))
    .one(&db)
    .await
    .expect("Failed to query client")
    .expect("Client with host 'fray' not found");

  let server_state = ServerState::build(db);

  // Set up mock client tunnel
  let tunnel_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind tunnel listener");
  let tunnel_addr = tunnel_listener.local_addr().expect("tunnel local addr");
  let client_tunnel_stream = TcpStream::connect(tunnel_addr).await.expect("connect tunnel");
  let (server_tunnel_stream, _) = tunnel_listener.accept().await.expect("accept tunnel");

  let (_, server_tunnel_writer) = tokio::io::split(server_tunnel_stream);
  let frame_writer = FrameWriter::new(server_tunnel_writer);
  server_state.add_client("fray".to_string(), frame_writer, client.id).await;

  let mut frame_reader = FrameReader::new(client_tunnel_stream);

  // Set up mock public HTTP client
  let public_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind public listener");
  let public_addr = public_listener.local_addr().expect("public local addr");
  let mut mock_http_client = TcpStream::connect(public_addr).await.expect("connect public");
  let (public_stream, _) = public_listener.accept().await.expect("accept public");

  let (tx, rx) = tokio::sync::mpsc::channel(100);

  let state_clone = server_state.clone();
  let public_task = tokio::spawn(async move {
    handle_public(public_stream, tx, rx, state_clone).await
  });

  // Send request bytes with headers and a body in a single write
  let request_bytes = b"POST /api/v1/test HTTP/1.1\r\nHost: fray.localhost\r\nContent-Type: text/plain\r\nContent-Length: 13\r\n\r\nhello, world!";
  mock_http_client.write_all(request_bytes).await.expect("write request bytes");
  mock_http_client.flush().await.expect("flush");
  mock_http_client.shutdown().await.expect("shutdown write half");

  // Read OpenStream frame
  let open_frame = frame_reader.read_frame().await.expect("read open stream frame");
  assert_eq!(open_frame.kind, FrameType::OpenStream, "Expected OpenStream frame first");
  let session_id = open_frame.id;

  // Read Data frames from tunnel until all data sent by handle_public is received
  let mut received_bytes = Vec::new();
  while let Ok(Ok(frame)) = tokio::time::timeout(Duration::from_millis(500), frame_reader.read_frame()).await {
    if frame.kind == FrameType::Data {
      received_bytes.extend_from_slice(&frame.payload);
    }
    if received_bytes.len() >= request_bytes.len() {
      // Allow a brief moment to check if duplicate bytes continue to arrive
      while let Ok(Ok(extra_frame)) = tokio::time::timeout(Duration::from_millis(100), frame_reader.read_frame()).await {
        if extra_frame.kind == FrameType::Data {
          received_bytes.extend_from_slice(&extra_frame.payload);
        }
      }
      break;
    }
  }

  // Close session so handle_public cleans up and finishes
  let _ = server_state.remove_sender(session_id).await;
  let public_result = tokio::time::timeout(Duration::from_secs(1), public_task).await;
  assert!(public_result.is_ok(), "handle_public should complete cleanly");

  // Assert received bytes match sent request bytes exactly (no duplicates)
  assert_eq!(
    received_bytes.len(),
    request_bytes.len(),
    "Incoming request bytes were sent more than once! Expected {} bytes, received {} bytes.\nSent: {:?}\nReceived: {:?}",
    request_bytes.len(),
    received_bytes.len(),
    String::from_utf8_lossy(request_bytes),
    String::from_utf8_lossy(&received_bytes)
  );
  assert_eq!(
    received_bytes,
    request_bytes,
    "Incoming request payload does not match sent request!"
  );

  // Assert api_log stored in DB has exact non-duplicated request payload
  let logs = entity::api_log::Entity::find()
    .filter(entity::api_log::Column::ClientId.eq(client.id))
    .all(&server_state.db)
    .await
    .expect("query api logs");
  assert_eq!(logs.len(), 1, "Expected 1 api log entry in db");
  assert_eq!(
    logs[0].request_payload,
    request_bytes,
    "api_log request_payload in database does not match sent bytes!"
  );
}

#[tokio::test]
async fn test_handle_public_chunked_request_bytes_not_duplicated() {
  let db = setup_and_seed_db().await;
  let client = entity::client::Entity::find()
    .filter(entity::client::Column::Host.eq("fray"))
    .one(&db)
    .await
    .expect("Failed to query client")
    .expect("Client with host 'fray' not found");

  let server_state = ServerState::build(db);

  // Set up mock client tunnel
  let tunnel_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind tunnel listener");
  let tunnel_addr = tunnel_listener.local_addr().expect("tunnel local addr");
  let client_tunnel_stream = TcpStream::connect(tunnel_addr).await.expect("connect tunnel");
  let (server_tunnel_stream, _) = tunnel_listener.accept().await.expect("accept tunnel");

  let (_, server_tunnel_writer) = tokio::io::split(server_tunnel_stream);
  let frame_writer = FrameWriter::new(server_tunnel_writer);
  server_state.add_client("fray".to_string(), frame_writer, client.id).await;

  let mut frame_reader = FrameReader::new(client_tunnel_stream);

  // Set up mock public HTTP client
  let public_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind public listener");
  let public_addr = public_listener.local_addr().expect("public local addr");
  let mut mock_http_client = TcpStream::connect(public_addr).await.expect("connect public");
  let (public_stream, _) = public_listener.accept().await.expect("accept public");

  let (tx, rx) = tokio::sync::mpsc::channel(100);

  let state_clone = server_state.clone();
  let public_task = tokio::spawn(async move {
    handle_public(public_stream, tx, rx, state_clone).await
  });

  // Send request in separate chunks: headers first, then body chunks with delays
  let headers = b"POST /api/v1/chunked HTTP/1.1\r\nHost: fray.localhost\r\nContent-Length: 16\r\n\r\n";
  let body_part1 = b"chunked-";
  let body_part2 = b"payload!";

  let mut full_expected_request = Vec::new();
  full_expected_request.extend_from_slice(headers);
  full_expected_request.extend_from_slice(body_part1);
  full_expected_request.extend_from_slice(body_part2);

  mock_http_client.write_all(headers).await.expect("write headers");
  mock_http_client.flush().await.expect("flush headers");
  tokio::time::sleep(Duration::from_millis(20)).await;

  mock_http_client.write_all(body_part1).await.expect("write body part 1");
  mock_http_client.flush().await.expect("flush body part 1");
  tokio::time::sleep(Duration::from_millis(20)).await;

  mock_http_client.write_all(body_part2).await.expect("write body part 2");
  mock_http_client.flush().await.expect("flush body part 2");
  mock_http_client.shutdown().await.expect("shutdown write half");

  // Read OpenStream frame
  let open_frame = frame_reader.read_frame().await.expect("read open stream frame");
  assert_eq!(open_frame.kind, FrameType::OpenStream, "Expected OpenStream frame first");
  let session_id = open_frame.id;

  // Read Data frames from tunnel
  let mut received_bytes = Vec::new();
  while let Ok(Ok(frame)) = tokio::time::timeout(Duration::from_millis(500), frame_reader.read_frame()).await {
    if frame.kind == FrameType::Data {
      received_bytes.extend_from_slice(&frame.payload);
    }
    if received_bytes.len() >= full_expected_request.len() {
      while let Ok(Ok(extra_frame)) = tokio::time::timeout(Duration::from_millis(100), frame_reader.read_frame()).await {
        if extra_frame.kind == FrameType::Data {
          received_bytes.extend_from_slice(&extra_frame.payload);
        }
      }
      break;
    }
  }

  // Close session
  let _ = server_state.remove_sender(session_id).await;
  let public_result = tokio::time::timeout(Duration::from_secs(1), public_task).await;
  assert!(public_result.is_ok(), "handle_public should complete cleanly");

  assert_eq!(
    received_bytes.len(),
    full_expected_request.len(),
    "Chunked request bytes were sent more than once! Expected {} bytes, received {} bytes.\nSent: {:?}\nReceived: {:?}",
    full_expected_request.len(),
    received_bytes.len(),
    String::from_utf8_lossy(&full_expected_request),
    String::from_utf8_lossy(&received_bytes)
  );
  assert_eq!(
    received_bytes,
    full_expected_request,
    "Chunked request payload does not match sent request!"
  );
}