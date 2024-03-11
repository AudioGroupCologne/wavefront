use bevy::math::UVec2;

pub struct WRect {
    pub min: UVec2,
    pub max: UVec2,
}

impl WRect {
    /// x1 and y1 are part of the wall
    pub fn new(x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        WRect {
            min: UVec2 { x: x0, y: y0 },
            max: UVec2 { x: x1, y: y1 },
        }
    }

    pub fn center(&self) -> UVec2 {
        UVec2 {
            x: (self.min.x + self.max.x) / 2,
            y: (self.min.y + self.max.y) / 2,
        }
    }

    pub fn width(&self) -> u32 {
        self.max.x - self.min.x + 1
    }

    pub fn height(&self) -> u32 {
        self.max.y - self.min.y + 1
    }
}
