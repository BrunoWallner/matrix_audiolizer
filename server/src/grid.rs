#![allow(dead_code)]

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