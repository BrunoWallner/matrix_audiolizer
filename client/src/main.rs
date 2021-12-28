#![allow(unreachable_code)]
#![allow(dead_code)]

use std::io::prelude::*;
use std::net::TcpStream;

use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
use audioviz::spectrum::stream::{Stream, StreamController};
use audioviz::spectrum::config::{StreamConfig, ProcessorConfig};

use simple_logger::SimpleLogger;
use log::{info, error};
use gag::Gag;

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
    let _print_gag = Gag::stderr().unwrap(); // to get rid of alsa warnings
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


    sleep(Duration::from_millis(100));

    // getting audio capture
    println!("");
    '_capture_selection: loop {
        println!("Index\t| device name");
        println!("----------------------");
        for (index, dev) in devices.iter().enumerate() {
            println!("{}:\t {}", index, dev);
        }
        let device = input("select device by index (empty for default): ")
            .parse::<usize>();

        println!("");
        let device = match device {
            Ok(device) => devices[device].clone(),
            Err(_) => String::from("default")
        };
        // captures audio from system using cpal
        match Capture::init(CaptureConfig {
            device,
            ..Default::default()
        }) {
            Ok(capture) => {
                println!("capturing audio"); // log would create panic
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
            
                // unrachable but would be good to do
                let instruction = TcpInstruction::CloseConnection;
                stream.write(&[instruction.to_byte()]).unwrap();
            },
            Err(e) => {
                error!("failed to caputure audio: {:#?}\n", e); // log would create panic
            }
        };
    }

    //let capture = Capture::init(CaptureConfig::default()).unwrap();

    
    // continuous processing of data received from capture
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
