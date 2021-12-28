use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::sync::mpsc;
use std::thread;


pub enum StreamEvent {
    RequestData( mpsc::Sender<Vec<u8>> ),
    SendData( Vec<u8> ),
}

// 1 byte
enum TcpInstruction {
    Invalid,
    SendData,
    SendComplete,
    CloseConnection,
}
impl TcpInstruction {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => TcpInstruction::SendData,
            0x01 => TcpInstruction::SendComplete,
            0x02 => TcpInstruction::CloseConnection,
            _ => TcpInstruction::Invalid,
        }
    }
}

fn handle_client(mut stream: TcpStream, sender: mpsc::Sender<StreamEvent>) {
    'connection: loop {
        let mut data: Vec<u8> = Vec::new(); 
        loop {
            let mut instruction_buffer: [u8; 1] = [0; 1];
            match stream.read_exact(&mut instruction_buffer) {
                Ok(_) => (),
                Err(_) => break 'connection,
            };
            match TcpInstruction::from_byte(instruction_buffer[0]) {
                    TcpInstruction::Invalid => (),
                    TcpInstruction::SendComplete => {
                        sender.clone().send(StreamEvent::SendData(data.clone()))
                            .unwrap();
                        data.clear();
                    },
                    TcpInstruction::SendData => {
                        let mut buffer: [u8; 16] = [0; 16];
                        match stream.read_exact(&mut buffer) {
                            Ok(_) => (),
                            Err(_) => break 'connection,
                        };
                        let mut b = trim_buffer(&buffer);
                        data.append(&mut b);
                    },
                    TcpInstruction::CloseConnection => {
                        break 'connection;
                    }
            }
        }
    }
}

pub fn init( ip: &str ) -> std::io::Result<mpsc::Sender<StreamEvent>> {
    let listener = TcpListener::bind(ip)?;

    let (stream_sender, stream_receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut data: Vec<u8> = Vec::new();
        loop {
            match stream_receiver.recv() {
                Ok(event) => {
                    match event {
                        StreamEvent::SendData(d) => data = d,
                        StreamEvent::RequestData(sender) => {
                            sender.send(data.clone()).unwrap()
                        },
                    }
                },
                Err(_) => (),
            }
        }
    });

    let s_s = stream_sender.clone();
    thread::spawn(move || {
        // accept connections and process them serially
        for stream in listener.incoming() {
            handle_client(stream.unwrap(), s_s.clone());
        }
    });
    Ok(stream_sender)
}

fn trim_buffer(buffer: &[u8; 16]) -> Vec<u8> {
    let length = buffer[0];
    let mut b: Vec<u8> = Vec::with_capacity(length as usize);
    for i in 1..=length {
        b.push(buffer[i as usize]);
    }
    b
}
