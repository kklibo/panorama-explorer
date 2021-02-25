use three_d::*;
pub use crate::LoadedImageMesh;
use crate::viewport_geometry::WorldCoords;

pub struct Photo<'a> {

    pub loaded_image_mesh: &'a LoadedImageMesh,
    scale: Mat4,
    translate: Mat4, //in world coord units

}

impl<'a> Photo<'a> {

    pub fn from_loaded_image_mesh(m: &LoadedImageMesh) -> Photo {

        let scale = Mat4::from_nonuniform_scale(m.pixel_width as f32,m.pixel_height as f32,1 as f32);
        let translate = Mat4::from_translation(Vec3::new(0f32, 0f32, 0f32));

        Photo {
            loaded_image_mesh: &m,
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