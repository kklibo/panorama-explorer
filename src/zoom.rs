
pub struct Zoom {
    pub scale: f64,
    pub value: u32,
    pub min: u32,
    pub max: u32,
    pub width_in_pixels: usize,
    pub height_in_pixels: usize,
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

    pub fn gl_units_width(&self) -> f64 {

        self.size_in_gl_units()
    }

    pub fn gl_units_height(&self) -> f64 {

        self.size_in_gl_units() / self.aspect_ratio_x_to_y()
    }

    pub fn gl_units_per_pixel(&self) -> f64 {
        if self.width_in_pixels == 0 {panic!("width_in_pixels = 0");}
        self.size_in_gl_units() / self.width_in_pixels as f64
    }

    fn size_in_gl_units(&self) -> f64 {
        2_u32.pow(self.value) as f64 * self.scale
    }

    fn aspect_ratio_x_to_y(&self) -> f64 {
        if self.height_in_pixels <= 0 {panic!("non-positive height in pixels");}
        let aspect = self.width_in_pixels as f64 / self.height_in_pixels as f64;
        if aspect <= 0.0 {panic!("non-positive aspect ratio");}
        aspect
    }
}

pub struct ScreenCoords {
    /// x location in screen units: [-0.5,0.5], positive is right
    pub x: f64,
    /// y location in screen units: [-0.5,0.5], positive is up
    pub y: f64,
}

pub struct PixelCoords {
    /// x location in pixels: [0.0, width], positive is right
    pub x: f64,
    /// y location in pixels: [0.0, height], positive is down
    pub y: f64,
}

pub struct WorldCoords {
    /// x location in world units: [left, right]
    pub x: f64,
    /// y location in world units: [bottom, top]
    pub y: f64,
}

pub fn pixel_to_screen(position: PixelCoords, screen_width_pixels: usize, screen_height_pixels: usize ) -> ScreenCoords {
    if screen_width_pixels  <= 0 {panic!("non-positive viewport width" );}
    if screen_height_pixels <= 0 {panic!("non-positive viewport height");}

    let x = position.x / screen_width_pixels as f64 - 0.5_f64;
    let y = 1_f64 - (position.y / screen_height_pixels as f64) - 0.5_f64;

    ScreenCoords{x,y}
}

//remove/replace this?
pub fn screen_to_world_at_origin(
    position: &ScreenCoords,
    screen_width_in_world_units: f64,
    screen_height_in_world_units: f64,
) -> WorldCoords {

    WorldCoords {
        x: screen_width_in_world_units * position.x,
        y: screen_height_in_world_units * position.y,
    }
}
