use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Grid {
    // how many 8x8 displays not pixels
    pub width: usize,
    pub height: usize,

    pub canvas: Vec<Vec<bool>>
}
impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let canvas =  vec![vec![false; width * 8]; height * 8];
        Grid {
            width,
            height,
            canvas,
        }
    }
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

    pub fn draw_point(&mut self, x: usize, y: usize) {
        if x < self.canvas[0].len() && y < self.canvas.len() {
            self.canvas[y][x] = true;
        }
    }

    pub fn gen_bars(&mut self, data: &[u8]) {
        for y in 0..self.height * 8 {
            for x in 0..self.width * 8 {
                if data.len() > x {
                    if data[x] >= y as u8 {
                        self.canvas[y as usize][x as usize] = true;
                    }
                }
            }
        }
    }
}