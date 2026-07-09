mod parser;

use crate::parser::{MinecraftReadExt, MinecraftWriteExt};
use std::env;
use std::io::{self, Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};

enum BulkWrite {
    I32(i32),
    String(String),
}

async fn write_bulk(
    w: &mut Vec<u8>,
    stream: &mut TcpStream,
    buf: Vec<BulkWrite>,
) -> io::Result<()> {
    for data in buf {
        match data {
            BulkWrite::String(s) => {
                stream
                    .write_mc_string(w, &s)
                    .await
                    .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
            }
            BulkWrite::I32(s) => {
                stream
                    .write_var_int(w, s)
                    .await
                    .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
            }
        }
    }

    Ok(())
}

pub async fn handle_client_connection(
    mut stream: TcpStream,
    target_addr:  &str,
) -> io::Result<()> {
    let _packet_length = stream.read_var_int().await?;
    let packet_id = stream.read_var_int().await?;

    if packet_id != 0x00 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to parse packet",
        ));
    }

    let protocol_version = stream.read_var_int().await?;
    let server_address = stream.read_mc_string().await?;

    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf).await?;

    let next_state = stream.read_var_int().await?;

    match TcpStream::connect(target_addr).await {
        Ok(mut target_stream) => {
            let mut packet_data: Vec<_> = Vec::new();
            let mut final_packet: Vec<u8> = Vec::new();

            let _ = write_bulk(
                &mut packet_data,
                &mut stream,
                vec![
                    BulkWrite::I32(packet_id),
                    BulkWrite::I32(protocol_version),
                    BulkWrite::String(server_address),
                ],
            )
            .await;

            packet_data.extend_from_slice(&port_buf);
            target_stream
                .write_var_int(&mut packet_data, next_state)
                .await?;

            target_stream
                .write_var_int(&mut final_packet, packet_data.len() as i32)
                .await?;
            final_packet.extend_from_slice(&packet_data);

            target_stream.flush().await?;
            target_stream.write_all(&final_packet).await?;

            if let Err(e) = copy_bidirectional(&mut stream, &mut target_stream).await {
                eprintln!("Transfer error: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to connect to target: {}", e),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25566").await?;

    println!("Server running in: 0.0.0.0:25565");

    match env::var("TARGET_ADDR") {
        Ok(target_addr) => loop {
            let (stream, _) = listener.accept().await?;
            let addr = target_addr.clone();

            tokio::spawn(async move { handle_client_connection(stream, &addr).await });
        },
        Err(e) => {
            return Err(Error::new(
                io::ErrorKind::InvalidData,
                "環境変数TARGET_ADDRが設定されていません",
            ));
        }
    }
}
