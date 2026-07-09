use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};

pub trait MinecraftWriteExt: AsyncWrite + Unpin {
    async fn write_var_int(&mut self, w: &mut Vec<u8>, value: i32) -> io::Result<()> {
        let mut ux = value as u32;
        loop {
            let mut b = (ux & 0x7F) as u8;
            ux >>= 7;
            if ux != 0 {
                b |= 0x80;
            }
            w.push(b);
            if ux == 0 {
                break;
            }
        }

        Ok(())
    }

    async fn write_mc_string(&mut self, w: &mut Vec<u8>, s: &str) -> io::Result<()> {
        self.write_var_int(w, s.len() as i32).await?;
        w.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

pub trait MinecraftReadExt: AsyncRead + Unpin {
    async fn read_var_int(&mut self) -> io::Result<i32> {
        let mut result = 0;
        let mut position = 0;
        let mut buf = [0u8; 1];

        loop {
            self.read_exact(&mut buf).await?;
            let current_byte = buf[0];
            result |= ((current_byte & 0x7F) as i32) << position;

            if (current_byte & 0x80) == 0 {
                break;
            }

            position += 7;
            if position >= 32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "VarInt too big"));
            }
        }

        Ok(result)
    }

    async fn read_mc_string(&mut self) -> io::Result<String> {
        let length = self.read_var_int().await? as usize;
        let mut buf = vec![0u8; length];
        self.read_exact(&mut buf).await?;
        String::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }
}

impl<R: AsyncRead + Unpin + ?Sized> MinecraftReadExt for R {}
impl<W: AsyncWrite + Unpin + ?Sized> MinecraftWriteExt for W {}
