use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt};

use crate::parser::{MinecraftWriteExt};
use std::{
    io::{self, Error},
};

pub async fn disconnect_client(mut stream: TcpStream) -> Result<(), Error> {
    let mut packet_data = Vec::new();
    let mut final_packet = Vec::new();
    let json_reason = format!(r#"{{"text":"{}"}}"#, "YOU DISCONNECTED");

    stream.write_var_int(&mut packet_data, 0x00).await?;
    stream.write_mc_string(&mut packet_data, &json_reason).await?;
    stream.write_var_int(&mut final_packet, packet_data.len() as i32).await?;
    final_packet.extend_from_slice(&packet_data);
    stream.write_all(&final_packet).await?;
    stream.flush().await?;

    stream.shutdown().await?;

    Ok(())
}

pub async fn send_motd_response(stream: &mut TcpStream) -> io::Result<()> {
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

    stream.write_var_int(&mut packet_data, 0x00).await?;
    stream.write_mc_string(&mut packet_data, json_motd).await?;

    stream.write_var_int(&mut final_packet, packet_data.len() as i32).await?;
    final_packet.extend_from_slice(&packet_data);

    stream.write_all(&final_packet).await?;
    stream.flush().await?;

    println!("MOTDを送信しました。");
    Ok(())
}
