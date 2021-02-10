
pub struct Zoom {
    pub scale: f64,
    pub value: u32,
    pub min: u32,
    pub max: u32,
}

impl Zoom {
    pub fn zoom_in(&mut self) {
        if self.value > self.min {
            self.value -= 1;
        }
    }
    pub fn zoom_out(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }

    pub fn gl_units_width(&self) -> f32 {

        self.size_in_gl_units() as f32
    }

    pub fn gl_units_height(&self, aspect_x_to_y: f32) -> f32 {

        if aspect_x_to_y <= 0.0 {panic!("non-positive aspect ratio");}
        self.size_in_gl_units() as f32 / aspect_x_to_y
    }

    pub fn gl_units_per_pixel(&self, width_in_pixels: usize) -> f64 {
        if width_in_pixels == 0 {panic!("width_in_pixels = 0");}
        self.size_in_gl_units() / width_in_pixels as f64
    }

    fn size_in_gl_units(&self) -> f64 {
        2_u32.pow(self.value) as f64 * self.scale
    }
}