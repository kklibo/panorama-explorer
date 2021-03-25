use std::rc::Rc;
use std::fmt::{Display,Formatter};

use three_d::*;
use cgmath::Deg;

pub use crate::entities::LoadedImageMesh;
use crate::viewport_geometry::WorldCoords;


pub struct Photo {

    pub loaded_image_mesh: Rc<LoadedImageMesh>,
    scale: Mat4,     //scales 1 (unwarped) pixel to 1 WorldCoords unit
    translate: Mat4, //in WorldCoords units
    rotate: Mat4,    //around photo center

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
        let rotate = Mat4::from_angle_z(Deg(0.0));

        Photo {
            loaded_image_mesh: m,
            scale,
            translate,
            rotate,
        }
    }

    pub fn to_world(&self) -> Mat4 {

        self.translate.concat(&self.rotate).concat(&self.scale)

    }

    pub fn set_translation(&mut self, center: WorldCoords) {

        self.translate = Mat4::from_translation(cgmath::Vector3::new(center.x as f32, center.y as f32, 0f32));
    }

    pub fn translation(&self) -> WorldCoords {

        let translate_vec = self.translate * Vec4::new(0.0, 0.0, 0.0, 1.0);
        WorldCoords { x: translate_vec.x as f64, y: translate_vec.y as f64 }
    }

    pub fn set_rotation(&mut self, angle: f32) {

        self.rotate = Mat4::from_angle_z(Deg(angle));
    }

    pub fn rotation(&self) -> f32 {
        let rotation_vec4 = self.rotate * Vec4::unit_x();
        let rotation_vec2 = Vec2::new(rotation_vec4.x, rotation_vec4.y);

        let angle: Deg<f32> = Vec2::unit_x().angle(rotation_vec2).into();
        angle.0
    }

    //todo: refine this interface?
    ///adjusts translation and rotation relative to their current values
    pub fn rotate_around_point(&mut self, angle: f32, point: WorldCoords) {

        //In this function, all translations and vectors are in WorldCoords units

       //adjust translation
        //get photo center
        let photo_center = self.translate * Vec4::unit_w();

        //get offset vector: image center to rotation point
        let rotation_point = Vec4::new(point.x as f32, point.y as f32, 0.0, 0.0);
        let to_rotation_point = rotation_point - photo_center;

        //add it to translate
        self.translate =
        self.translate.concat(&Mat4::from_translation(to_rotation_point.truncate()));

        //rotate offset vector by angle
        let rotate_by_angle = Mat4::from_angle_z(Deg(angle));
        let rotated = rotate_by_angle * to_rotation_point;

        //subtract it from translate
        self.translate =
        self.translate.concat(&Mat4::from_translation(-rotated.truncate()));


       //adjust rotation
        //add rotation to current rotate matrix
        self.rotate =
        self.rotate.concat(&rotate_by_angle);

    }



    //todo: update for rotation
    //todo: apply distortion (option?)
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

//todo: update for rotation
//todo: apply distortion (option?)
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