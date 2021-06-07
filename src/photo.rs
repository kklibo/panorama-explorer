use std::rc::Rc;
use std::fmt::{Display,Formatter};

use three_d::{Mat4,Texture,InnerSpace};

use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;

pub use crate::entities::LoadedImageMesh;
use crate::viewport_geometry::{WorldCoords, PixelCoords};
use crate::world_rectangle::{WorldRectangle,LocalCoords};


pub struct Photo {

    pub loaded_image_mesh: Rc<LoadedImageMesh>,

    ///this Photo's world space orientation:
    ///* scales 1 (unwarped) pixel to 1 WorldCoords unit
    ///* translates center from world origin in WorldCoords units
    ///* rotates around photo center
    orientation: WorldRectangle,

}

//todo: make this complete?
impl Serialize for Photo {

    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {

        let mut state = serializer.serialize_struct("Photo", 2)?;
        state.serialize_field("translate", &self.orientation.translate)?;
        state.serialize_field("rotate", &self.orientation.rotate)?;
        state.end()
    }
}

impl Display for Photo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "scale: {:?}", self.orientation.scale)?;
        writeln!(f, "translate: {:?}", self.orientation.translate)?;
        Ok(())
    }
}

impl Photo {

    pub fn from_loaded_image_mesh(m: Rc<LoadedImageMesh>) -> Self {

        let orientation = WorldRectangle::new(m.texture_2d.width() as f32,m.texture_2d.height() as f32);

        Self {
            loaded_image_mesh: m,
            orientation,
        }
    }

    pub fn set_from_json_serde_string(&mut self, s: &str) -> Result<(), Box<dyn std::error::Error>> {

        #[derive(Deserialize)]
        struct SavedFields {
            translate: Mat4,
            rotate: Mat4,
        }

        let saved_fields: SavedFields = serde_json::from_str(s)?;
        self.orientation.translate = saved_fields.translate;
        self.orientation.rotate = saved_fields.rotate;

        Ok(())
    }

    pub fn orientation(&self) -> &WorldRectangle {
        &self.orientation
    }

    pub fn set_translation(&mut self, center: WorldCoords) {

        self.orientation.set_translation(center)
    }

    pub fn set_rotation(&mut self, angle: f32) {

        self.orientation.set_rotation(angle)
    }

    pub fn rotate_around_point(&mut self, angle: f32, point: WorldCoords) {

        self.orientation.rotate_around_point(angle, point)
    }

    /// gets the WorldCoords location of pixel coords in this photo
    pub fn world_coords(&self, pixel_coords: PixelCoords) -> WorldCoords {

        Self::world_coords_impl(&self.orientation, pixel_coords)
    }

    fn world_coords_impl(world_rectangle: &WorldRectangle, pixel_coords: PixelCoords) -> WorldCoords {

        let local_coords = Self::local_coords(world_rectangle, pixel_coords);

        world_rectangle.world_coords(local_coords)
    }

    fn local_coords(world_rectangle: &WorldRectangle, pixel_coords: PixelCoords) -> LocalCoords {

        let width = world_rectangle.scale.x.magnitude() as f64;
        let height = world_rectangle.scale.y.magnitude() as f64;


        let local_x =
        if width == 0.0 {
            //if width is somehow 0, center on origin
            0.0
        }
        else {
            //scale to width = 1
            let x = pixel_coords.x / width;
            //center on origin
            let x = x - 0.5;
            x
        };

        let local_y =
        if height == 0.0 {
            //if height is somehow 0, center on origin
            0.0
        }
        else {
            //scale to width = 1
            let y = pixel_coords.y / height;
            //center on origin
            let y = y - 0.5;
            //flip y-coords to positive = up
            let y = -y;
            y
        };

        LocalCoords{x: local_x, y: local_y}
    }
}


#[cfg(test)]
mod test {
    use super::*;


    #[test]
    fn world_coords_test() {

        //at origin, no rotation
        {
            let mut orientation = WorldRectangle::new(200.0, 100.0);
            orientation.set_rotation(0.0);
            orientation.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            //top left corner
            {
                let pixel_coords = PixelCoords { x: 0.0, y: 0.0 };
                let world_coords = Photo::world_coords_impl(&orientation, pixel_coords);

                assert_eq!(world_coords, WorldCoords { x: -100.0, y: 50.0 });
            }

            //bottom right corner
            {
                let pixel_coords = PixelCoords { x: 200.0, y: 100.0 };
                let world_coords = Photo::world_coords_impl(&orientation, pixel_coords);

                assert_eq!(world_coords, WorldCoords { x: 100.0, y: -50.0 });
            }
        }

        //rotated + translated
        {
            let mut orientation = WorldRectangle::new(200.0, 100.0);
            orientation.set_rotation(90.0);
            let x = 2000.0;
            let y = 1000.0;
            orientation.set_translation(WorldCoords {x,y});

            //top left corner
            {
                let pixel_coords = PixelCoords { x: 0.0, y: 0.0 };
                let world_coords = Photo::world_coords_impl(&orientation, pixel_coords);

                assert_eq!(world_coords, WorldCoords { x: x - 50.0, y: y - 100.0 });
            }

            //bottom right corner
            {
                let pixel_coords = PixelCoords { x: 200.0, y: 100.0 };
                let world_coords = Photo::world_coords_impl(&orientation, pixel_coords);

                assert_eq!(world_coords, WorldCoords { x: x + 50.0, y: y + 100.0 });
            }
        }

    }
}