use std::rc::Rc;

use three_d::*;
pub use crate::LoadedImageMesh;
use crate::viewport_geometry::WorldCoords;
use std::fmt::{Display,Formatter};

pub struct Photo {

    pub loaded_image_mesh: Rc<LoadedImageMesh>,
    scale: Mat4,
    translate: Mat4, //in world coord units

}

impl Display for Photo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "scale: {:?}", self.scale)?;
        writeln!(f, "translate: {:?}", self.translate)?;
        Ok(())
    }
}

impl Photo {

    pub fn from_loaded_image_mesh(m: Rc<LoadedImageMesh>) -> Photo {

        let scale = Mat4::from_nonuniform_scale(m.texture_2d.width() as f32,m.texture_2d.height() as f32,1 as f32);
        let translate = Mat4::from_translation(Vec3::new(0f32, 0f32, 0f32));

        Photo {
            loaded_image_mesh: m,
            scale,
            translate,
        }
    }

    pub fn to_world(&self) -> Mat4 {

        self.translate.concat(&self.scale)

    }

    pub fn set_translation(&mut self, center: WorldCoords) {

        self.translate = Mat4::from_translation(cgmath::Vector3::new(center.x as f32, center.y as f32, 0f32));
    }

    pub fn translation(&self) -> WorldCoords {

        let translate_vec = self.translate * Vec4::new(0.0, 0.0, 0.0, 1.0);
        WorldCoords { x: translate_vec.x as f64, y: translate_vec.y as f64 }
    }

    pub fn contains(&self, point: WorldCoords) -> bool {

        let bottom_left_corner_world_coords = self.to_world() * Vec4::new(-0.5,-0.5,0.0, 1.0);
        let   top_right_corner_world_coords = self.to_world() * Vec4::new( 0.5, 0.5,0.0, 1.0);

        log::info!("contains: bottom left: {}, {}", bottom_left_corner_world_coords.x, bottom_left_corner_world_coords.y);
        log::info!("            top right: {}, {}", top_right_corner_world_coords.x, top_right_corner_world_coords.y);
        log::info!("                  scale: {:?}", self.scale);
        log::info!("              translate: {:?}", self.translate);


        bottom_left_corner_world_coords.x <= point.x as f32 &&
        point.x as f32 <= top_right_corner_world_coords.x &&

        bottom_left_corner_world_coords.y <= point.y as f32 &&
        point.y as f32 <= top_right_corner_world_coords.y
    }
}

pub fn convert_photo_px_to_world(v: Vec3, m: &Photo) -> Mat4 {

    let to_bottom_left = Mat4::from_translation(Vec3::new(-0.5,-0.5,0.0));
    let to_v = Mat4::from_translation(v);

    let translation_and_scale =
        m.translate
        //flip y-coords
        .concat(&Mat4::from_nonuniform_scale(1.0, -1.0, 1.0))
        .concat(&to_v)
        .concat(&m.scale)
        //scaled to photo space
        .concat(&to_bottom_left);


    let mut just_translation = Mat4::identity();
    just_translation.w = translation_and_scale.w;

    just_translation
}