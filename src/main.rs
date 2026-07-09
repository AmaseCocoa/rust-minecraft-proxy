mod parser;

use crate::parser::{MinecraftReadExt, MinecraftWriteExt};
use std::io::{self};
use tokio::io::{AsyncReadExt, AsyncWriteExt, copy_bidirectional};
use tokio::net::{TcpListener, TcpStream};


pub async fn handle_client_connection(mut stream: TcpStream) -> io::Result<()> {
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

    let server_port = u16::from_be_bytes(port_buf);
    println!("アドレス: {}:{}", server_address, server_port);

    let target_addr = format!("{}:{}", server_address, 25565); // server_port
    let next_state = stream.read_var_int().await?;

    match TcpStream::connect(target_addr).await {
        Ok(mut target_stream) => {
            let mut packet_data: Vec<_> = Vec::new();
            let mut final_packet: Vec<u8> = Vec::new();

            target_stream.write_var_int(&mut packet_data, packet_id).await?;
            target_stream.write_var_int(&mut packet_data, protocol_version).await?;
            target_stream.write_mc_string(&mut packet_data, &server_address).await?;
            packet_data.extend_from_slice(&port_buf);
            target_stream.write_var_int(&mut packet_data, next_state).await?;

            target_stream.write_var_int(&mut final_packet, packet_data.len() as i32).await?;
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

    loop {
        let (stream, _) = listener.accept().await?;

        tokio::spawn(async move { handle_client_connection(stream).await });
    }
}
