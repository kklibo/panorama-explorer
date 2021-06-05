use std::rc::Rc;
use std::fmt::{Display,Formatter};

use three_d::{Vec3,Vec4,Mat4,Texture};

use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;

pub use crate::entities::LoadedImageMesh;
use crate::viewport_geometry::{WorldCoords, PixelCoords};
use crate::world_rectangle::WorldRectangle;


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

        let v = Vec3::new(pixel_coords.x as f32, pixel_coords.y as f32, 0.0);
        let m = world_rectangle.convert_local_to_world(v);

        let out_v = m * Vec4::unit_w();

        WorldCoords{x: out_v.x as f64, y: out_v.y as f64}
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