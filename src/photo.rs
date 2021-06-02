use std::rc::Rc;
use std::fmt::{Display,Formatter};

use three_d::{Mat4,Texture};

use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;

pub use crate::entities::LoadedImageMesh;
use crate::viewport_geometry::WorldCoords;
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

    pub fn from_loaded_image_mesh(m: Rc<LoadedImageMesh>) -> Photo {

        let orientation = WorldRectangle::new(m.texture_2d.width() as f32,m.texture_2d.height() as f32);

        Photo {
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
}
