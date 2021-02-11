
pub struct ViewportGeometry {
    pub zoom_scale: f64,
    pub zoom_value: u32,
    pub zoom_min: u32,
    pub zoom_max: u32,
    pub width_in_pixels: usize,
    pub height_in_pixels: usize,
}

impl ViewportGeometry {
    pub fn zoom_in(&mut self) {
        if self.zoom_value > self.zoom_min {
            self.zoom_value -= 1;
        }
    }
    pub fn zoom_out(&mut self) {
        if self.zoom_value < self.zoom_max {
            self.zoom_value += 1;
        }
    }

    pub fn width_in_world_units(&self) -> f64 {

        self.size_in_world_units()
    }

    pub fn height_in_world_units(&self) -> f64 {

        self.size_in_world_units() / self.aspect_ratio_x_to_y()
    }

    pub fn world_units_per_pixel(&self) -> f64 {
        if self.width_in_pixels == 0 {panic!("width_in_pixels = 0");}
        self.size_in_world_units() / self.width_in_pixels as f64
    }

    pub fn convert_pixel_to_screen(&self, position: PixelCoords) -> ScreenCoords {
        if self.width_in_pixels  <= 0 {panic!("non-positive viewport width" );}
        if self.height_in_pixels <= 0 {panic!("non-positive viewport height");}

        let x = position.x / self.width_in_pixels as f64 - 0.5_f64;
        let y = 1_f64 - (position.y / self.height_in_pixels as f64) - 0.5_f64;

        ScreenCoords{x,y}
    }

    //remove/replace this?
    pub fn convert_screen_to_world_at_origin(&self, position: &ScreenCoords) -> WorldCoords {
        WorldCoords {
            x: self.width_in_world_units() * position.x,
            y: self.height_in_world_units() * position.y,
        }
    }

    fn size_in_world_units(&self) -> f64 {
        2_u32.pow(self.zoom_value) as f64 * self.zoom_scale
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
