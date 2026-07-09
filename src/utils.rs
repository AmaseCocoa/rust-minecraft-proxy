use crate::parser::{MinecraftWriteExt};
use std::{
    io::{self, Error, Write},
    net::{TcpStream},
};

pub fn disconnect_client(mut stream: TcpStream) -> Result<(), Error> {
    let mut packet_data = Vec::new();
    let mut final_packet = Vec::new();
    let json_reason = format!(r#"{{"text":"{}"}}"#, "YOU DISCONNECTED");

    stream.write_var_int(&mut packet_data, 0x00);
    stream.write_mc_string(&mut packet_data, &json_reason);
    stream.write_var_int(&mut final_packet, packet_data.len() as i32);
    final_packet.extend_from_slice(&packet_data);
    stream.write_all(&final_packet)?;
    stream.flush()?;

    stream.shutdown(std::net::Shutdown::Both)?;

    Ok(())
}

pub fn send_motd_response(mut stream: &TcpStream) -> io::Result<()> {
    let json_motd = r#"{
        "version": {
            "name": "1.20",
            "protocol": 763
        },
        "players": {
            "max": 20,
            "online": 3
        },
        "description": {
            "text": "§aRustで動く§rカスタムMinecraftサーバー\n§7接続テスト中..."
        }
    }"#;

    let mut packet_data = Vec::new();
    let mut final_packet = Vec::new();

    stream.write_var_int(&mut packet_data, 0x00);
    stream.write_mc_string(&mut packet_data, json_motd);

    stream.write_var_int(&mut final_packet, packet_data.len() as i32);
    final_packet.extend_from_slice(&packet_data);

    stream.write_all(&final_packet)?;
    stream.flush()?;

    println!("MOTDを送信しました。");
    Ok(())
}
