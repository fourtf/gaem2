#[derive(Debug, Copy, Clone, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Rect {
        return Rect {
            x: x,
            y: y,
            width: width,
            height: height,
        };
    }

    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    pub fn move_right(&mut self, right: f64) {
        self.x = right - self.width;
    }

    pub fn move_bottom(&mut self, bottom: f64) {
        self.y = bottom - self.height;
    }
}
