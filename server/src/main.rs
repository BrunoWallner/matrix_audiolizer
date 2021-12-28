use gpio_cdev::{Chip, LineRequestFlags, LineHandle, Error};
use std::thread::sleep;
use std::time::Duration;
use std::sync::mpsc;

mod matrix;
use matrix::*;

mod grid;
use grid::Grid;

mod stream;

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
        8,
    );

    let ip = "0.0.0.0:4225";
    let stream_sender = stream::init(ip).unwrap();
    println!("server listening on {}", ip);

    matrix.init().unwrap();

    for addr in 0..8 {
        matrix.set_intensity(addr, 0x01).unwrap();
    }

    loop {
        let (s, r) = mpsc::channel();
        stream_sender.clone().send(stream::StreamEvent::RequestData(s)).unwrap();
        let data = r.recv().unwrap();

        let mut grid = Grid::new(4, 2);
        grid.gen_bars(&data);

        matrix.draw_grid(grid).unwrap();
  
        sleep(Duration::from_millis(16));
    }

    //matrix.power_off().unwrap();
}
