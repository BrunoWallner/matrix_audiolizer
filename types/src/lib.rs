use serde::{Serialize, Deserialize};

pub mod grid;

pub enum TcpInstruction {
    Invalid,
    SendGrid,
    GridComplete,
    CloseConnection,
    RequestConfiguration,
}

// POV of client
impl TcpInstruction {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => TcpInstruction::SendGrid,
            0x01 => TcpInstruction::GridComplete,
            0x02 => TcpInstruction::CloseConnection,
            0x03 => TcpInstruction::RequestConfiguration,
            _ => TcpInstruction::Invalid,
        }
    }
    pub fn to_byte(&self) -> u8 {
        match self {
            TcpInstruction::SendGrid => 0x00,
            TcpInstruction::GridComplete => 0x01,
            TcpInstruction::CloseConnection => 0x02,
            TcpInstruction::RequestConfiguration => 0x03,
            TcpInstruction::Invalid => 0xFF,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MatrixConfiguration {
    pub width: u8,
    pub height: u8
}
impl MatrixConfiguration {
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        match bincode::serialize(self) {
            Ok(s) => Some(s),
            Err(_) => None
        }
    }
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bincode::deserialize(bytes) {
            Ok(d) => Some(d),
            Err(_) => None,
        }
    }
}

pub fn vec_to_buffers(vec: &[u8]) -> Vec<Vec<u8>> {
    let mut chunk_buffer: Vec<Vec<u8>> = Vec::new();
    for chunk in vec.chunks(255) {
        let mut b: Vec<u8> = vec![0; 256];
        for i in 0..chunk.len() {
            b[i + 1] = chunk[i];
        }
        b[0] = chunk.len() as u8;

        chunk_buffer.push(b);      
    }
    chunk_buffer
}

pub fn trim_buffer(buffer: &[u8; 256]) -> Vec<u8> {
    let length = buffer[0];
    let mut b: Vec<u8> = Vec::with_capacity(length as usize);
    for i in 1..=length {
        b.push(buffer[i as usize]);
    }
    b
}
