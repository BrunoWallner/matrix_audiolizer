#![allow(dead_code)]
use gpio_cdev::{LineHandle, Error};
use crate::grid::Grid;

// Maximum number of displays connected in series supported
const MAX_DISPLAYS: u8 = 8;

// Digits per addr
const MAX_DIGITS: u8 = 8;

// Possible command register values on the addr chip.
#[derive(Clone, Copy)]
pub enum Command {
    Noop = 0x00,
    Digit0 = 0x01,
    Digit1 = 0x02,
    Digit2 = 0x03,
    Digit3 = 0x04,
    Digit4 = 0x05,
    Digit5 = 0x06,
    Digit6 = 0x07,
    Digit7 = 0x08,
    DecodeMode = 0x09,
    Intensity = 0x0A,
    ScanLimit = 0x0B,
    Power = 0x0C,
    DisplayTest = 0x0F,
}

pub struct Matrix {
    pub cs: LineHandle,
    pub data: LineHandle,
    pub clock: LineHandle,
    pub devices: u8,
    pub buffer: [u8; MAX_DISPLAYS as usize * 2]
}
impl Matrix {
    pub fn new(
        cs: LineHandle,
        data: LineHandle,
        clock: LineHandle,
        devices: u8,
    ) -> Self {
        Matrix { cs, data, clock, devices, buffer: [0; MAX_DISPLAYS as usize * 2] }
    }

    pub fn init(&mut self) -> Result<(), Error> {
        for device in 0..self.devices {
            self.write_raw(device, Command::ScanLimit as u8, 0x07)?; // to scan for 8 digits or in this case columns
            self.write_raw(device, Command::DecodeMode as u8, 0x00)?; // No Decode Mode for matrix
            self.write_raw(device, Command::DisplayTest as u8, 0x00)?;
            self.write_raw(device, Command::Power as u8, 0x01)?; // powers on display
        }

        Ok(())
    }
    pub fn power_off(&mut self) -> Result<(), Error>  {
        for addr in 0..self.devices {
            self.write_raw(addr, Command::Power as u8, 0x00)?;
        }
        Ok(())
    }
    pub fn send_command(&mut self, addr: u8, command: Command, value: u8) -> Result<(), Error> {
        self.write_raw(addr, command as u8, value)?;
        Ok(())
    }
    pub fn draw_raw(&mut self, addr: u8, data: [u8; 8]) -> Result<(), Error> {
        let mut digit: u8 = 1; // column
        for b in data {
            self.write_raw(addr, digit, b)?;
            digit += 1;
        }
        Ok(())
    }

    pub fn draw_grid(&mut self, grid: Grid) -> Result<(), Error> {
        /*
        for addr_offset in 0..grid.height {
            for addr in 0..grid.width {
                let addr_with_offset = addr + (addr_offset * grid.width);

                let mut bytes: [u8; 8] = [0; 8];
                if addr <= self.devices as usize && addr + 8 < grid.width * 8 {
                    for y in 0..8 {
                        // builds u8 out of bools
                        let mut byte: u8 = 0b00000000;
                        for i in 0..8 {
                            // shifts bits to left
                            byte <<= 1;
                            // sets first bit to true
                            if grid.canvas[y + (addr_offset * 8)][addr * 8 + i] {
                                byte |= 0b00000001;
                            }
                        }
                        bytes[y] = byte;
                    }
                    self.draw_raw(addr_with_offset as u8, bytes)?;
                }
            } 
        }
        */
        // 8x4 matrices chained like this: =]
        if grid.width * grid.height <= self.devices as usize {
            for display in 0..8 {//grid.width * grid.height {
                // display section of grid
                let start_y = (display / grid.width) * 8;
                let end_y = start_y + 8;
                let mut start_x = (display % grid.width) * 8;
                let mut end_x = start_x + 8;

                // for weird chaining
                let first_row = {(display / grid.width) == 1};
                if first_row {
                    start_x = (grid.width - ((display % grid.width) + 1) ) * 8;
                    end_x = start_x + 8;
                }

                let mut section: [[bool; 8]; 8] = [[false; 8]; 8];
                for y in start_y..end_y {
                    let rel_y = y - start_y;
                    for x in start_x..end_x {
                        let rel_x = x - start_x;
                        // with additional rotation in first row because of messed up wiring
                        if first_row {
                            section[7 - rel_y][7 - rel_x] = grid.canvas[y][x];
                        } else {
                            section[rel_y][rel_x] = grid.canvas[y][x]; 
                        }
                    }
                }
                
                let mut bytes: [u8; 8] = [0; 8];
                for y in 0..8 {
                    let mut byte: u8 = 0x00;
                    for x in 0..8 {
                        // shifts bits to left
                        byte <<= 1;
                        // sets first bit to true
                        if section[y][x] {
                            byte |= 0b00000001;
                        }
                    }
                    bytes[y] = byte;
                }
                self.draw_raw(display as u8, bytes)?;
                // */
            }
        }
        Ok(())
    }

    pub fn clear_display(&mut self, addr: u8) -> Result<(), Error> {
        for col in 1..=MAX_DIGITS {
            self.write_raw(addr, col, 0x00)?;
        }
        Ok(())
    }

    /// range 0 to 15
    pub fn set_intensity(&mut self, addr: u8, intesity: u8) -> Result<(), Error> {
        self.write_raw(addr, Command::Intensity as u8, intesity)?;
        Ok(())
    }

    fn write_raw(&mut self, addr: u8, header: u8, data: u8) -> Result<(), Error> {
        let offset = addr as usize * 2;
        let max_bytes = self.devices * 2;
        self.buffer = [0; MAX_DISPLAYS as usize * 2];

        self.buffer[offset] = header;
        self.buffer[offset + 1] = data;

        self.cs.set_value(0)?;
        for b in 0..max_bytes as usize {
            let value = self.buffer[b];

            for i in 0..8 {
                if value & (1 << (7 - i)) > 0 {
                    self.data.set_value(1)?;
                } else {
                    self.data.set_value(0)?;
                }

                self.clock.set_value(1)?;
                self.clock.set_value(0)?;
            }
        }
        self.cs.set_value(1)?;

        Ok(())
    }
}
