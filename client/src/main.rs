#![allow(unreachable_code)]
#![allow(dead_code)]
use audioviz::audio_capture::{config::Config as CaptureConfig, capture::Capture};
use audioviz::spectrum::stream::{Stream as AudioStream, StreamController};
use audioviz::spectrum::config::{StreamConfig, ProcessorConfig};

use gag::Gag;
use types::grid::Grid;
use std::{thread::sleep, time::Duration, process::exit};

mod stream;
use stream::Stream;

fn main() {
    let _print_gag = Gag::stderr().unwrap(); // to get rid of alsa warnings

    let ip = input("ip: ");
    let mut stream = match Stream::connect(&ip) {
        Some(s) => {
            s
        },
        None => {
            exit(1);
        }
    };
    let matrix_config = stream.get_matrix_configuration().unwrap();

    let devices = match Capture::fetch_devices() {
        Ok(d) => d,
        Err(e) => {
            println!("unable to fetch devices: {:?}", e);
            exit(1);
        }
    };

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
                let audio = AudioStream::init_with_capture(&capture, StreamConfig {
                    gravity: Some(50.0),
                    fft_resolution: 2048,
                    processor: ProcessorConfig {
                        frequency_bounds: [30, 5_00],
                        volume: 2.0,
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
                        data.insert(0, freq.volume as u8)
                    }

                    let mut grid = Grid::new(matrix_config.width as usize, matrix_config.height as usize);
                    grid.gen_bars(&data);

                    stream.send_grid(&grid).unwrap();

                    sleep(Duration::from_millis(15));
                }
            
            
            },
            Err(e) => {
                println!("invalid device: {:?}", e);
                exit(1);
            }
        };
    }
}

use std::io::Write;

fn input(print: &str) -> String {
    print!("{}", print);
    std::io::stdout().flush().unwrap();
    let mut input = String::new();

    std::io::stdin().read_line(&mut input)
        .ok()
        .expect("Couldn't read line");
        
    input.trim().to_string()
}