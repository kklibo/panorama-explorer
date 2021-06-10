
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use std::ops::Add;

#[derive(Debug, Copy, Clone)]
pub struct ViewportGeometry {
    pub camera_position: WorldCoords,
    pub zoom_scale: f64,
    pub zoom_value: u32,
    pub zoom_min: u32,
    pub zoom_max: u32,
    width_in_pixels: NonZeroUsize,
    height_in_pixels: NonZeroUsize,
}

#[derive(Debug, Copy, Clone)]
pub enum PixelDimensionError {
    ZeroWidth,
    ZeroHeight,
    ZeroWidthAndHeight
}

impl Display for PixelDimensionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PixelDimensionError {}

impl ViewportGeometry {

    pub fn try_new(
        camera_position: WorldCoords,
        zoom_scale: f64,
        zoom_value: u32,
        zoom_min: u32,
        zoom_max: u32,
        width_in_pixels: usize,
        height_in_pixels: usize,
    ) -> Result<ViewportGeometry, PixelDimensionError> {

        let (width_in_pixels, height_in_pixels) = Self::check_pixel_dimensions(width_in_pixels, height_in_pixels)?;

        Ok( ViewportGeometry {
            camera_position, zoom_scale, zoom_value, zoom_min, zoom_max, width_in_pixels, height_in_pixels
        })
    }

    pub fn set_pixel_dimensions(
        &mut self,
        width_in_pixels: usize,
        height_in_pixels: usize,
    ) -> Result<(), PixelDimensionError> {

        let (width_in_pixels, height_in_pixels) = Self::check_pixel_dimensions(width_in_pixels, height_in_pixels)?;

        self.width_in_pixels = width_in_pixels;
        self.height_in_pixels = height_in_pixels;
        Ok(())
    }

    fn check_pixel_dimensions(width_in_pixels: usize, height_in_pixels: usize) -> Result<(NonZeroUsize, NonZeroUsize), PixelDimensionError> {

        match (NonZeroUsize::new(width_in_pixels), NonZeroUsize::new(height_in_pixels)) {
            (None,    Some(_)) => Err(PixelDimensionError::ZeroWidth),
            (Some(_), None   ) => Err(PixelDimensionError::ZeroHeight),
            (None,    None   ) => Err(PixelDimensionError::ZeroWidthAndHeight),
            (Some(w), Some(h)) => Ok((w,h)),
        }
    }

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

    #[allow(dead_code)]
    pub fn width_in_pixels(&self) -> NonZeroUsize {
        self.width_in_pixels
    }

    #[allow(dead_code)]
    pub fn height_in_pixels(&self) -> NonZeroUsize {
        self.height_in_pixels
    }

    pub fn width_in_world_units(&self) -> f64 {

        self.size_in_world_units()
    }

    pub fn height_in_world_units(&self) -> f64 {

        self.size_in_world_units() / self.aspect_ratio_x_to_y()
    }

    pub fn world_units_per_pixel(&self) -> f64 {
        self.size_in_world_units() / self.width_in_pixels.get() as f64
    }

    pub fn convert_pixel_to_screen(&self, position: &PixelCoords) -> ScreenCoords {

        let x = position.x / self.width_in_pixels.get() as f64 - 0.5_f64;
        let y = 1_f64 - (position.y / self.height_in_pixels.get() as f64) - 0.5_f64;

        ScreenCoords{x,y}
    }

    pub fn pixels_to_world(&self, pixel_coords: &PixelCoords) -> WorldCoords {

        let screen_coords = self.convert_pixel_to_screen(pixel_coords);

        self.convert_screen_to_world_at_origin(&screen_coords) + self.camera_position
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
        self.width_in_pixels.get() as f64 / self.height_in_pixels.get() as f64
    }
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct ScreenCoords {
    /// x location in screen units: [-0.5,0.5], positive is right
    pub x: f64,
    /// y location in screen units: [-0.5,0.5], positive is up
    pub y: f64,
}

#[derive(Debug, PartialOrd, PartialEq)]
pub struct PixelCoords {
    /// x location in pixels: [0.0, width], positive is right
    pub x: f64,
    /// y location in pixels: [0.0, height], positive is down
    pub y: f64,
}

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub struct WorldCoords {
    /// x location in world units: [left, right]
    pub x: f64,
    /// y location in world units: [bottom, top]
    pub y: f64,
}

impl Add for WorldCoords {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        WorldCoords{ x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use assert_matches::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn try_new_test() {

        assert_matches!(
            ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
                1_f64, 10_u32, 1_u32, 15_u32,
                0_usize,
                100_usize,
            ),
            Err(PixelDimensionError::ZeroWidth)
        );

        assert_matches!(
            ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
                1_f64, 10_u32, 1_u32, 15_u32,
                100_usize,
                0_usize,
            ),
            Err(PixelDimensionError::ZeroHeight)
        );

        assert_matches!(
            ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
                1_f64, 10_u32, 1_u32, 15_u32,
                0_usize,
                0_usize,
            ),
            Err(PixelDimensionError::ZeroWidthAndHeight)
        );

        let res = ViewportGeometry::try_new(
                WorldCoords{x: 123.4, y: 567.8},
                1_f64, 10_u32, 1_u32, 15_u32,
                100_usize,
                200_usize,
            ).unwrap();

        assert_eq!(res.camera_position, WorldCoords{x: 123.4, y: 567.8});
        assert_eq!(res.zoom_scale, 1_f64);
        assert_eq!(res.zoom_value, 10_u32);
        assert_eq!(res.zoom_min, 1_u32);
        assert_eq!(res.zoom_max, 15_u32);
        assert_eq!(res.width_in_pixels, NonZeroUsize::new(100_usize).unwrap());
        assert_eq!(res.height_in_pixels, NonZeroUsize::new(200_usize).unwrap());
    }

    #[test]
    fn zoom_in_test() {

        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            1.0, 0,0,10,
            100,200).unwrap();

        {
            //failed zoom: at lower limit
            let mut v = v;
            v.zoom_value = 10;
            v.zoom_min = 10;
            v.zoom_in();
            assert_eq!(v.zoom_value, 10);
        }

        {
            //failed zoom: at lower limit of 0
            let mut v = v;
            v.zoom_value = 0;
            v.zoom_min = 0;
            v.zoom_in();
            assert_eq!(v.zoom_value, 0);
        }

        {
            //successful zoom
            let mut v = v;
            v.zoom_value = 10;
            v.zoom_min = 0;
            v.zoom_in();
            assert_eq!(v.zoom_value, 9);
        }

        {
            //successful zoom to 0
            let mut v = v;
            v.zoom_value = 1;
            v.zoom_min = 0;
            v.zoom_in();
            assert_eq!(v.zoom_value, 0);
        }
    }

    #[test]
    fn zoom_out_test() {

        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            1.0, 0,0,10,
            100,200).unwrap();

        {
            //failed zoom: at upper limit
            let mut v = v;
            v.zoom_value = 10;
            v.zoom_max = 10;
            v.zoom_out();
            assert_eq!(v.zoom_value, 10);
        }

        {
            //failed zoom: at upper limit of u32::MAX
            let mut v = v;
            v.zoom_value = u32::MAX;
            v.zoom_max = u32::MAX;
            v.zoom_out();
            assert_eq!(v.zoom_value, u32::MAX);
        }

        {
            //successful zoom
            let mut v = v;
            v.zoom_value = 0;
            v.zoom_max = 10;
            v.zoom_out();
            assert_eq!(v.zoom_value, 1);
        }

        {
            //successful zoom to u32::MAX
            let mut v = v;
            v.zoom_value = u32::MAX - 1;
            v.zoom_max = u32::MAX;
            v.zoom_out();
            assert_eq!(v.zoom_value, u32::MAX);
        }
    }

    #[test]
    fn width_in_world_units_test() {
        let zoom_value = 10 as u32;
        let zoom_scale = 2 as f64;
        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            zoom_scale, zoom_value, 0, 10,
            400, 200).unwrap();

        assert_approx_eq!(v.width_in_world_units(), 2048 as f64);
    }

    #[test]
    fn height_in_world_units_test() {
        let zoom_value = 10 as u32;
        let zoom_scale = 2 as f64;
        let width_in_pixels = 400 as usize;
        let height_in_pixels = 200 as usize;
        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            zoom_scale, zoom_value, 0, 10,
            width_in_pixels, height_in_pixels).unwrap();

        assert_approx_eq!(v.height_in_world_units(), 1024 as f64);
    }

    #[test]
    fn world_units_per_pixel_test() {
        let zoom_value = 10 as u32;
        let zoom_scale = 2 as f64;
        let width_in_pixels = 1024 as usize;
        let height_in_pixels = 512 as usize;
        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            zoom_scale, zoom_value, 0, 10,
            width_in_pixels, height_in_pixels).unwrap();

        assert_approx_eq!(v.world_units_per_pixel(), 2 as f64);
    }

    #[test]
    fn convert_pixel_to_screen_test() {
        let zoom_value = 10 as u32;
        let zoom_scale = 2 as f64;
        let width_in_pixels = 1024 as usize;
        let height_in_pixels = 512 as usize;
        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            zoom_scale, zoom_value, 0, 10,
            width_in_pixels, height_in_pixels).unwrap();

        {
            let pixel_coords = PixelCoords { x: 0.0, y: 0.0 };
            let ScreenCoords { x, y } = v.convert_pixel_to_screen(&pixel_coords);
            assert_approx_eq!(x, -0.5);
            assert_approx_eq!(y, 0.5);
        }

        {
            let pixel_coords = PixelCoords { x: 1024.0, y: 512.0 };
            let ScreenCoords { x, y } = v.convert_pixel_to_screen(&pixel_coords);
            assert_approx_eq!(x, 0.5);
            assert_approx_eq!(y, -0.5);
        }
    }

    #[test]
    fn convert_screen_to_world_at_origin_test() {
        let zoom_value = 10 as u32;
        let zoom_scale = 2 as f64;
        let width_in_pixels = 400 as usize;
        let height_in_pixels = 200 as usize;
        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            zoom_scale, zoom_value, 0, 10,
            width_in_pixels, height_in_pixels).unwrap();

        let screen_coords = ScreenCoords{x: 0.0, y: 0.0};
        let WorldCoords{x, y} = v.convert_screen_to_world_at_origin( &screen_coords);
        assert_approx_eq!(x, 0.0);
        assert_approx_eq!(y, 0.0);

        let screen_coords = ScreenCoords{x: -0.5, y: 0.5};
        let WorldCoords{x, y} = v.convert_screen_to_world_at_origin( &screen_coords);
        assert_approx_eq!(x, -1024.0);
        assert_approx_eq!(y, 512.0);
    }

    #[test]
    fn size_in_world_units_test() {
        let zoom_value = 10 as u32;
        let zoom_scale = 2 as f64;
        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            zoom_scale, zoom_value, 0, 10,
            400, 200).unwrap();

        assert_approx_eq!(v.size_in_world_units(), 2048 as f64);
    }

    #[test]
    fn aspect_ratio_x_to_y_test() {
        let (width, height) = (100_usize, 200_usize);
        let v = ViewportGeometry::try_new(
                WorldCoords{x: 0.0, y: 0.0},
            1.0, 0, 0, 10,
            width, height).unwrap();

        assert_approx_eq!(v.aspect_ratio_x_to_y(), 0.5);
    }


}