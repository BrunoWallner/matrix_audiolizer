use std::io::prelude::*;
use std::net::TcpStream;

use types::{
    TcpInstruction, MatrixConfiguration,
    trim_buffer, vec_to_buffers,
    grid::Grid,
};

pub struct Stream {
    stream: TcpStream
}
impl Stream {
    pub fn connect(ip: &str) -> Option<Self> {
        let stream = match TcpStream::connect(ip.clone()) {
            Ok(s) => s,
            Err(_) => return None,
        };
        Some(
            Self {stream}
        )
    }

    pub fn get_matrix_configuration(&mut self) -> Option<MatrixConfiguration> {
        self.stream.write_all(&[TcpInstruction::RequestConfiguration.to_byte()]).unwrap();
        let mut data: Vec<u8> = Vec::new();
        'configuration: loop {
            let mut instruction_buffer: [u8; 1] = [0; 1];
            self.stream.read_exact(&mut instruction_buffer).unwrap();
            match TcpInstruction::from_byte(instruction_buffer[0]) {
                TcpInstruction::SendGrid => {
                    let mut buffer: [u8; 256] = [0; 256];
                    self.stream.read_exact(&mut buffer).unwrap();
                    let d = trim_buffer(&buffer);
                    for d in d {
                        data.push(d);
                    }
                }
                TcpInstruction::GridComplete => break 'configuration,
                _ => (),
            }
        }  
        MatrixConfiguration::from_bytes(&data)
    }

    pub fn send_grid(&mut self, grid: &Grid) -> Result<(), std::io::Error> {
        let data = grid.to_bytes().unwrap_or(vec![0]);
        let data = vec_to_buffers(&data);

        for d in data {
            // sends data to server
            let instruction = TcpInstruction::SendGrid;
            self.stream.write_all(&[instruction.to_byte()])?;
            self.stream.write_all(&d)?;
        }

        let instruction = TcpInstruction::GridComplete;
        self.stream.write_all(&[instruction.to_byte()])?;

        Ok(())
    }

    pub fn close_connection(&mut self) -> Result<(), std::io::Error> {
        let instruction = TcpInstruction::CloseConnection;
        self.stream.write(&[instruction.to_byte()])?;

        Ok(())
    }
}