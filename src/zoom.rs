
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct ViewportGeometry {
    pub zoom_scale: f64,
    pub zoom_value: u32,
    pub zoom_min: u32,
    pub zoom_max: u32,
    width_in_pixels: NonZeroUsize,
    height_in_pixels: NonZeroUsize,
}

#[derive(Debug)]
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
        zoom_scale: f64,
        zoom_value: u32,
        zoom_min: u32,
        zoom_max: u32,
        width_in_pixels: usize,
        height_in_pixels: usize,
    ) -> Result<ViewportGeometry, PixelDimensionError> {

        let (width_in_pixels, height_in_pixels) = Self::check_pixel_dimensions(width_in_pixels, height_in_pixels)?;

        Ok( ViewportGeometry {
            zoom_scale, zoom_value, zoom_min, zoom_max, width_in_pixels, height_in_pixels
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

    pub fn width_in_world_units(&self) -> f64 {

        self.size_in_world_units()
    }

    pub fn height_in_world_units(&self) -> f64 {

        self.size_in_world_units() / self.aspect_ratio_x_to_y()
    }

    pub fn world_units_per_pixel(&self) -> f64 {
        self.size_in_world_units() / self.width_in_pixels.get() as f64
    }

    pub fn convert_pixel_to_screen(&self, position: PixelCoords) -> ScreenCoords {

        let x = position.x / self.width_in_pixels.get() as f64 - 0.5_f64;
        let y = 1_f64 - (position.y / self.height_in_pixels.get() as f64) - 0.5_f64;

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
        self.width_in_pixels.get() as f64 / self.height_in_pixels.get() as f64
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

#[cfg(test)]
mod test {

    use super::*;
    use assert_matches::*;

    #[test]
    fn try_new_test() {

        assert_matches!(
            ViewportGeometry::try_new(
                1_f64, 10_u32, 1_u32, 15_u32,
                0_usize,
                100_usize,
            ),
            Err(PixelDimensionError::ZeroWidth)
        );

        assert_matches!(
            ViewportGeometry::try_new(
                1_f64, 10_u32, 1_u32, 15_u32,
                100_usize,
                0_usize,
            ),
            Err(PixelDimensionError::ZeroHeight)
        );

        assert_matches!(
            ViewportGeometry::try_new(
                1_f64, 10_u32, 1_u32, 15_u32,
                0_usize,
                0_usize,
            ),
            Err(PixelDimensionError::ZeroWidthAndHeight)
        );

        assert_matches!(
            ViewportGeometry::try_new(
                1_f64, 10_u32, 1_u32, 15_u32,
                100_usize,
                100_usize,
            ),
            Ok(_)
        );
    }


}