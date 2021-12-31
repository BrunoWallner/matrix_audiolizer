use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::sync::mpsc;
use std::thread;

use types::{
    TcpInstruction,
    vec_to_buffers, trim_buffer,
};

pub enum StreamEvent {
    RequestData( mpsc::Sender<Vec<u8>> ),
    SendGrid( Vec<u8> ),
}


fn handle_client(mut stream: TcpStream, sender: mpsc::Sender<StreamEvent>, configuration: (u8, u8) /* width, height */ ) {
    'connection: loop {
        let mut grid_bytes: Vec<u8> = Vec::new(); 
        loop {
            let mut instruction_buffer: [u8; 1] = [0; 1];
            match stream.read_exact(&mut instruction_buffer) {
                Ok(_) => (),
                Err(_) => break 'connection,
            };
            match TcpInstruction::from_byte(instruction_buffer[0]) {
                    TcpInstruction::Invalid => (),
                    TcpInstruction::GridComplete => {
                        sender.clone().send(StreamEvent::SendGrid(grid_bytes.clone()))
                            .unwrap();
                        grid_bytes.clear();
                    },
                    TcpInstruction::SendGrid => {
                        let mut buffer: [u8; 256] = [0; 256];
                        match stream.read_exact(&mut buffer) {
                            Ok(_) => (),
                            Err(_) => break 'connection,
                        };
                        let mut b = trim_buffer(&buffer);
                        grid_bytes.append(&mut b);
                    },
                    TcpInstruction::CloseConnection => {
                        break 'connection;
                    }
                    TcpInstruction::RequestConfiguration => {
                        let data = vec_to_buffers(&[configuration.0, configuration.1]);
                        for b in data {
                            stream.write_all(&[TcpInstruction::SendGrid.to_byte()]).unwrap();
                            stream.write_all(&b).unwrap();
                        }
                        stream.write_all(&[TcpInstruction::GridComplete.to_byte()]).unwrap();
                    }
            }
        }
    }
}

pub fn init( ip: &str, configuration: (u8, u8) /* width, height */ ) -> std::io::Result<mpsc::Sender<StreamEvent>> {
    let listener = TcpListener::bind(ip)?;

    let (stream_sender, stream_receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut data: Vec<u8> = Vec::new();
        loop {
            match stream_receiver.recv() {
                Ok(event) => {
                    match event {
                        StreamEvent::SendGrid(d) => data = d,
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
            let s_s = s_s.clone();
            thread::spawn(move || {
                handle_client(stream.unwrap(), s_s.clone(), configuration);
            });
        }
    });
    Ok(stream_sender)
}