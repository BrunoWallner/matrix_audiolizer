use gpio_cdev::{Chip, LineRequestFlags, LineHandle, Error};
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc;

mod matrix;
use matrix::*;

use types::grid::Grid;

mod stream;

/* Number of 8x8 matrices chained together, NOT PIXELS */
const WIDTH: u8 = 4;
const HEIGHT: u8 = 2;

// CS Data Clock
fn get_pins(cs: u32, data: u32, clock: u32) -> Result<(LineHandle, LineHandle, LineHandle), Error> {
    let mut chip = Chip::new("/dev/gpiochip0")?;
    
    let cs = chip
        .get_line(cs)? // 24 8
        .request(LineRequestFlags::OUTPUT, 0, "chip-select")?;

    let data = chip
        .get_line(data)? // 19 10
        .request(LineRequestFlags::OUTPUT, 0, "mosi")?;

    let clock = chip
        .get_line(clock)? // 23 11
        .request(LineRequestFlags::OUTPUT, 0, "clock")?;

    Ok( (cs, data, clock) )
}

fn main() {
    let (cs, data, clock) = get_pins(2, 10, 11).unwrap();
    let mut matrix = Matrix::new(
        cs,
        data,
        clock,
        WIDTH * HEIGHT,
    );

    let ip = "0.0.0.0:4225";
    let stream_sender = stream::init(ip, (WIDTH, HEIGHT)).unwrap();
    println!("server listening on {}", ip);

    matrix.init().unwrap();

    loop {
        let (s, r) = mpsc::channel();
        stream_sender.clone().send(stream::StreamEvent::RequestData(s)).unwrap();
        let data = r.recv().unwrap();
        let grid = Grid::from_bytes(&data).unwrap_or(Grid::new(WIDTH as usize, HEIGHT as usize));

        matrix.draw_grid(grid).unwrap();
  
        sleep(Duration::from_millis(16));
    }

    //matrix.power_off().unwrap();
}

