use std::rc::Rc;
use std::fmt::{Display,Formatter};

use three_d::{Vec2,Vec3,Vec4,Mat4,Texture,Transform,InnerSpace,SquareMatrix};
use cgmath::Deg;

use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;

pub use crate::entities::LoadedImageMesh;
use crate::viewport_geometry::WorldCoords;


pub struct Photo {

    pub loaded_image_mesh: Rc<LoadedImageMesh>,
    scale: Mat4,     //scales 1 (unwarped) pixel to 1 WorldCoords unit
    translate: Mat4, //in WorldCoords units
    rotate: Mat4,    //around photo center

}

//todo: make this complete?
impl Serialize for Photo {

    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {

        let mut state = serializer.serialize_struct("Photo", 2)?;
        state.serialize_field("translate", &self.translate)?;
        state.serialize_field("rotate", &self.rotate)?;
        state.end()
    }
}

impl Display for Photo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "scale: {:?}", self.scale)?;
        writeln!(f, "translate: {:?}", self.translate)?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Copy, Clone)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
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

    pub fn set_from_json_serde_string(&mut self, s: &str) -> Result<(), Box<dyn std::error::Error>> {

        #[derive(Deserialize)]
        struct SavedFields {
            translate: Mat4,
            rotate: Mat4,
        }

        let saved_fields: SavedFields = serde_json::from_str(s)?;
        self.translate = saved_fields.translate;
        self.rotate = saved_fields.rotate;

        Ok(())
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

    ///returns a corner's WorldCoords location as a Vec2
    fn corner_worldcoords_vec2(&self, corner: Corner) -> Vec2 {

        let v = match corner {
            Corner::TopLeft => self.to_world() * Vec4::new(-0.5,0.5,0.0, 1.0),
            Corner::TopRight => self.to_world() * Vec4::new(0.5,0.5,0.0, 1.0),
            Corner::BottomLeft => self.to_world() * Vec4::new(-0.5,-0.5,0.0, 1.0),
            Corner::BottomRight => self.to_world() * Vec4::new(0.5,-0.5,0.0, 1.0),
        };

        Vec2::new(v.x, v.y)
    }

    ///returns true IFF the point is on the 'inside' side of this edge or collinear with the edge
    fn is_inside(&self, point: WorldCoords, edge: Edge) -> bool {

        let point = Vec2::new(point.x as f32, point.y as f32);

        let corner_on_edge = match edge {
            Edge::Bottom | Edge::Left => self.corner_worldcoords_vec2(Corner::BottomLeft),
            Edge::Top | Edge::Right => self.corner_worldcoords_vec2(Corner::TopRight),
        };

        let inward_normal_point = match edge {
            Edge::Bottom | Edge::Right => self.corner_worldcoords_vec2(Corner::TopLeft),
            Edge::Top | Edge::Left => self.corner_worldcoords_vec2(Corner::BottomRight),
        };

        let to_point = point - corner_on_edge;
        let inward_normal = inward_normal_point - corner_on_edge;

        inward_normal.dot(to_point) >= 0.0
    }


    pub fn corner(&self, corner: Corner) -> WorldCoords {

        let v = self.corner_worldcoords_vec2(corner);

        WorldCoords{x: v.x as f64, y: v.y as f64}
    }

    ///true IFF the point is within the undistorted edges of this photo
    pub fn contains(&self, point: WorldCoords) -> bool {

        self.is_inside(point, Edge::Left) &&
        self.is_inside(point, Edge::Right) &&
        self.is_inside(point, Edge::Top) &&
        self.is_inside(point, Edge::Bottom)
    }

    ///returns a matrix to translate a location in this photo (in pixel units) to a world location (in worldcoord units)
    pub fn convert_photo_px_to_world(&self, v: Vec3) -> Mat4 {

        let to_bottom_left = Mat4::from_translation(Vec3::new(-0.5,-0.5,0.0));
        let to_v = Mat4::from_translation(v);

        let transformation =
            self.translate
            .concat(&self.rotate)
            //flip y-coords
            .concat(&Mat4::from_nonuniform_scale(1.0, -1.0, 1.0))
            .concat(&to_v)
            .concat(&self.scale)
            //scaled to photo space
            .concat(&to_bottom_left);


        let mut just_translation = Mat4::identity();
        just_translation.w = transformation.w;

        just_translation
    }
}
