use std::io::prelude::*;
use std::net::TcpStream;

use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
use audioviz::spectrum::stream::{Stream, StreamController};
use audioviz::spectrum::config::{StreamConfig, ProcessorConfig};

use simple_logger::SimpleLogger;
use log::{info, trace, warn, error};

use std::{thread::sleep, time::Duration, process::exit};

enum TcpInstruction {
    Invalid,
    SendData,
    SendComplete,
    CloseConnection,
}
impl TcpInstruction {
    fn to_byte(&self) -> u8 {
        match self {
            TcpInstruction::SendData => 0x00,
            TcpInstruction::SendComplete => 0x01,
            TcpInstruction::CloseConnection => 0x02,
            TcpInstruction::Invalid => 0xFF,
        }
    }
}

fn main() {
    SimpleLogger::new().init().unwrap();

    let ip = input("ip: ");
    let mut stream = match TcpStream::connect(ip.clone()) {
        Ok(s) => {
            info!( "connected to: {}", ip );
            s
        },
        Err(e) => {
            error!( "could not connect to {}: {}", ip, e );
            exit(1);
        }
    };

    let devices = match Capture::fetch_devices() {
        Ok(d) => d,
        Err(e) => {
            error!("failed to fetch devices: {:#?}", e);
            exit(1);
        }
    };
    for (index, dev) in devices.iter().enumerate() {
        println!("{}: {}", index, dev);
    }

    // captures audio from system using cpal
    let capture = match Capture::init(CaptureConfig::default()) {
        Ok(c) => {
            info!("capturing audio");
            c
        },
        Err(e) => {
            println!("failed to caputure audio: {:#?}", e); // log would create panic
            exit(1);
        }
    };
    
    
 
    // continuous processing of data received from capture
    let audio = Stream::init_with_capture(&capture, StreamConfig {
        gravity: Some(200.0),
        processor: ProcessorConfig {
            frequency_bounds: [30, 15_000],
            ..Default::default()
        },
        ..Default::default()
    });
    let audio_controller: StreamController = audio.get_controller();

    audio_controller.set_resolution(32);

    loop {
        let freqs = audio_controller.get_frequencies();
        let mut data: Vec<u8> = Vec::new();
        for freq in freqs {
            data.insert(0, (freq.volume.sqrt() * 4.0) as u8)
        }


        let data = vec_to_buffers(&data);

        for d in data {
            // sends data to server
            let instruction = TcpInstruction::SendData;
            stream.write_all(&[instruction.to_byte()]).unwrap();
            stream.write_all(&d).unwrap();
        }

        let instruction = TcpInstruction::SendComplete;
        stream.write_all(&[instruction.to_byte()]).unwrap();

        sleep(Duration::from_millis(16));
    }

    let instruction = TcpInstruction::CloseConnection;
    stream.write(&[instruction.to_byte()]).unwrap();
}

fn input(print: &str) -> String {
    print!("{}", print);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();

    std::io::stdin().read_line(&mut input)
        .ok()
        .expect("Couldn't read line");
        
    input.trim().to_string()
}

fn vec_to_buffers(vec: &[u8]) -> Vec<Vec<u8>> {
    let mut chunk_buffer: Vec<Vec<u8>> = Vec::new();
    for chunk in vec.chunks(15) {
        let mut b: Vec<u8> = vec![0; 16];
        for i in 0..chunk.len() {
            b[i + 1] = chunk[i];
        }
        b[0] = chunk.len() as u8;

        chunk_buffer.push(b);      
    }
    chunk_buffer
}
