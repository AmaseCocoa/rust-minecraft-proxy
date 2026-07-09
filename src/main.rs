mod parser;
mod utils;

use crate::parser::{MinecraftReadExt, MinecraftWriteExt};
use crate::utils::{send_motd_response, disconnect_client};
use std::{
    io::{self, Error, Read, Write},
    net::{TcpListener, TcpStream},
};



fn handle_status_request(mut stream: TcpStream) -> Result<(), Error> {
    let _req_len = stream.read_var_int()?;
    let req_id = stream.read_var_int()?;

    if req_id != 0x00 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Status Request"));
    }

    let _ = send_motd_response(&stream);

    let _ping_len = stream.read_var_int()?;
    let ping_id = stream.read_var_int()?;
    if ping_id == 0x01 {
        let mut time_payload = [0u8; 8];
        let mut pong_data = Vec::new();
        let mut final_pong = Vec::new();

        stream.read_exact(&mut time_payload)?;
        stream.write_var_int(&mut pong_data, 0x01);
        stream.write_var_int(&mut final_pong, pong_data.len() as i32);

        pong_data.extend_from_slice(&time_payload);

        stream.write_all(&final_pong)?;
        stream.flush()?;
    }

    Ok(())
}

pub fn handle_client_connection(mut stream: TcpStream) -> io::Result<()> {
    let _packet_length = stream.read_var_int()?;
    let packet_id = stream.read_var_int()?;

    if packet_id != 0x00 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse packet"));
    }

    let _protocol_version = stream.read_var_int()?;
    let server_address = stream.read_mc_string()?;

    let mut port_buf = [0u8; 2];
    stream.read_exact(&mut port_buf)?;

    let num_be = u16::from_be_bytes(port_buf);
    println!("アドレス: {}:{}", server_address, num_be);

    let next_state = stream.read_var_int()?;

    match next_state {
        1 => {
            handle_status_request(stream)?;
        }
        2 => {
            disconnect_client(stream)?;
        }
        _ => {
            println!("未知のNext State（状態）です: {}", next_state);
        }
    }

    Ok(())
}


fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:25565")?;

    println!("Server running in: 0.0.0.0:25565");

    for stream in listener.incoming() {
        let _ = handle_client_connection(stream?);
    }

    Ok(())
}
