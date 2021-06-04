
use std::fmt::{Display,Formatter};

use three_d::{Vec2,Vec3,Vec4,Mat4,Transform,InnerSpace,SquareMatrix};
use cgmath::{Deg, AbsDiffEq};

use serde::{Serialize, Deserialize, Serializer};
use serde::ser::SerializeStruct;

use crate::viewport_geometry::WorldCoords;

/// A rectangle in the worldspace, defined by a scale, rotation, and translation.
///
/// In the rectangle's local coordinate system:
/// * (0,0) is the center
/// * (+/-0.5, +/-0.5) are corners
#[derive(Debug, PartialEq)]
pub struct WorldRectangle {

    pub scale: Mat4,     //in WorldCoords units
    pub translate: Mat4, //in WorldCoords units
    pub rotate: Mat4,    //rotation around center

}

impl AbsDiffEq<WorldRectangle> for WorldRectangle {
    type Epsilon = f32;

    fn default_epsilon() -> Self::Epsilon {
        //f32::default_epsilon()
        0.00001
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {

        self.scale.abs_diff_eq(&other.scale, epsilon) &&
        self.translate.abs_diff_eq(&other.translate, epsilon) &&
        self.rotate.abs_diff_eq(&other.rotate, epsilon)
    }
}

impl Serialize for WorldRectangle {

    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error> where
        S: Serializer {

        let mut state = serializer.serialize_struct("WorldRectangle", 3)?;
        state.serialize_field("scale", &self.scale)?;
        state.serialize_field("translate", &self.translate)?;
        state.serialize_field("rotate", &self.rotate)?;
        state.end()
    }
}

impl Display for WorldRectangle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "scale: {:?}", self.scale)?;
        writeln!(f, "translate: {:?}", self.translate)?;
        writeln!(f, "rotate: {:?}", self.rotate)?;
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

impl WorldRectangle {

    pub fn new(width: f32, height: f32) -> Self {

        let scale = Mat4::from_nonuniform_scale(width, height,1 as f32);
        let translate = Mat4::from_translation(Vec3::new(0f32, 0f32, 0f32));
        let rotate = Mat4::from_angle_z(Deg(0.0));

        Self {
            scale,
            translate,
            rotate,
        }
    }

    pub fn set_from_json_serde_string(&mut self, s: &str) -> Result<(), Box<dyn std::error::Error>> {

        #[derive(Deserialize)]
        struct SavedFields {
            scale: Mat4,
            translate: Mat4,
            rotate: Mat4,
        }

        let saved_fields: SavedFields = serde_json::from_str(s)?;
        self.scale = saved_fields.scale;
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
        //get rectangle center
        let rectangle_center = self.translate * Vec4::unit_w();

        //get offset vector: image center to rotation point
        let rotation_point = Vec4::new(point.x as f32, point.y as f32, 0.0, 0.0);
        let to_rotation_point = rotation_point - rectangle_center;

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

    ///true IFF the point is within the undistorted edges of this rectangle
    pub fn contains(&self, point: WorldCoords) -> bool {

        self.is_inside(point, Edge::Left) &&
        self.is_inside(point, Edge::Right) &&
        self.is_inside(point, Edge::Top) &&
        self.is_inside(point, Edge::Bottom)
    }

    ///returns a matrix to translate a location in this rectangle to a world location
    //todo: this uses pixel coords as input: refactor to elsewhere
    pub fn convert_local_to_world(&self, v: Vec3) -> Mat4 {

        let to_bottom_left = Mat4::from_translation(Vec3::new(-0.5,-0.5,0.0));
        let to_v = Mat4::from_translation(v);

        let transformation =
            self.translate
            .concat(&self.rotate)
            //flip y-coords
            .concat(&Mat4::from_nonuniform_scale(1.0, -1.0, 1.0))
            .concat(&to_v)
            .concat(&self.scale)
            //scaled to rectangle size
            .concat(&to_bottom_left);


        let mut just_translation = Mat4::identity();
        just_translation.w = transformation.w;

        just_translation
    }
}


#[cfg(test)]
mod test {

    use super::*;
    use std::error::Error;
    use cgmath::assert_abs_diff_eq;

    #[test]
    fn serde_test() -> Result<(), Box<dyn Error>> {

        let mut serde_in = WorldRectangle::new(300.0, 200.0);
        serde_in.set_rotation(10.0);
        serde_in.set_translation(WorldCoords{x: 50.0, y: 65.0});

        let serde_string = serde_json::to_string(&serde_in)?;

        let mut serde_out = WorldRectangle::new(0.0, 0.0);
        serde_out.set_from_json_serde_string(&serde_string)?;

        assert_eq!(serde_in, serde_out);

        Ok(())
    }

    #[test]
    fn to_world_test() -> Result<(), Box<dyn Error>> {

        //at origin, no rotation
        {
            let mut world_rectangle = WorldRectangle::new(200.0, 100.0);
            world_rectangle.set_rotation(0.0);
            world_rectangle.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            let m = world_rectangle.to_world();

            assert_abs_diff_eq!(m * Vec4::new(0.0, 0.0, 0.0, 1.0) , Vec4::new(0.0, 0.0, 0.0, 1.0) );
            assert_abs_diff_eq!(m * Vec4::new(1.0, 1.0, 0.0, 1.0) , Vec4::new(200.0, 100.0, 0.0, 1.0) );
        }

        //rotation + translation
        {
            let mut world_rectangle = WorldRectangle::new(200.0, 100.0);
            world_rectangle.set_rotation(90.0);
            world_rectangle.set_translation(WorldCoords { x: 2000.0, y: 1000.0 });

            let m = world_rectangle.to_world();

            assert_abs_diff_eq!(m * Vec4::new(0.0, 0.0, 0.0, 1.0) , Vec4::new(2000.0, 1000.0, 0.0, 1.0) );
            assert_abs_diff_eq!(m * Vec4::new(1.0, 1.0, 0.0, 1.0) , Vec4::new(1900.0, 1200.0, 0.0, 1.0) );
        }
        Ok(())
    }

    #[test]
    fn translation_test() -> Result<(), Box<dyn Error>> {

        {
            let mut world_rectangle = WorldRectangle::new(300.0, 200.0);
            let translation = WorldCoords { x: 50.0, y: 65.0 };
            world_rectangle.set_translation(translation);
            assert_eq!(translation, world_rectangle.translation());
        }

        {
            let mut world_rectangle = WorldRectangle::new(300.0, 200.0);
            let translation = WorldCoords { x: -50.0, y: -65.0 };
            world_rectangle.set_translation(translation);
            assert_eq!(translation, world_rectangle.translation());
        }

        Ok(())
    }

    #[test]
    fn rotation_test() -> Result<(), Box<dyn Error>> {

        {
            let mut world_rectangle = WorldRectangle::new(300.0, 200.0);
            let rotation_deg = 25.0;
            world_rectangle.set_rotation(rotation_deg);
            assert_eq!(rotation_deg, world_rectangle.rotation());
        }

        {
            let mut world_rectangle = WorldRectangle::new(300.0, 200.0);
            let rotation_deg = -25.0;
            world_rectangle.set_rotation(rotation_deg);
            assert_eq!(rotation_deg, world_rectangle.rotation());
        }

        Ok(())
    }

    #[test]
    fn rotate_around_point_test() -> Result<(), Box<dyn Error>> {

        //no-op rotation at origin
        {
            let mut world_rectangle = WorldRectangle::new(300.0, 200.0);
            world_rectangle.set_rotation(0.0);
            world_rectangle.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            let mut after_rotation = WorldRectangle::new(300.0, 200.0);
            after_rotation.set_rotation(0.0);
            after_rotation.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            world_rectangle.rotate_around_point(0.0, WorldCoords{x: 0.0, y: 0.0});

            assert_abs_diff_eq!(world_rectangle, after_rotation)
        }

        //90deg rotation around origin
        {
            let mut world_rectangle = WorldRectangle::new(300.0, 200.0);
            world_rectangle.set_rotation(0.0);
            world_rectangle.set_translation(WorldCoords { x: 50.0, y: 50.0 });

            let mut after_rotation = WorldRectangle::new(300.0, 200.0);
            after_rotation.set_rotation(90.0);
            after_rotation.set_translation(WorldCoords { x: -50.0, y: 50.0 });

            world_rectangle.rotate_around_point(90.0, WorldCoords{x: 0.0, y: 0.0});

            assert_abs_diff_eq!(world_rectangle, after_rotation)
        }

        //180deg rotation away from origin
        {
            let mut world_rectangle = WorldRectangle::new(300.0, 200.0);
            world_rectangle.set_rotation(0.0);
            world_rectangle.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            let mut after_rotation = WorldRectangle::new(300.0, 200.0);
            after_rotation.set_rotation(180.0);
            after_rotation.set_translation(WorldCoords { x: 100.0, y: 100.0 });

            world_rectangle.rotate_around_point(180.0, WorldCoords{x: 50.0, y: 50.0});

            assert_abs_diff_eq!(world_rectangle, after_rotation)
        }

        Ok(())
    }

    #[test]
    fn corner_test() -> Result<(), Box<dyn Error>> {

        //at origin, no rotation
        {
            let mut world_rectangle = WorldRectangle::new(200.0, 100.0);
            world_rectangle.set_rotation(0.0);
            world_rectangle.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            assert_eq!(world_rectangle.corner(Corner::TopLeft), WorldCoords{x: -100.0, y: 50.0});
            assert_eq!(world_rectangle.corner(Corner::TopRight), WorldCoords{x: 100.0, y: 50.0});
            assert_eq!(world_rectangle.corner(Corner::BottomLeft), WorldCoords{x: -100.0, y: -50.0});
            assert_eq!(world_rectangle.corner(Corner::BottomRight), WorldCoords{x: 100.0, y: -50.0});
        }

        //rotated + translated
        {
            let mut world_rectangle = WorldRectangle::new(200.0, 100.0);
            world_rectangle.set_rotation(90.0);
            let x = 2000.0;
            let y = 1000.0;
            world_rectangle.set_translation(WorldCoords {x,y});

            assert_eq!(world_rectangle.corner(Corner::TopLeft), WorldCoords{x: x - 50.0, y: y - 100.0});
            assert_eq!(world_rectangle.corner(Corner::TopRight), WorldCoords{x: x - 50.0, y: y + 100.0});
            assert_eq!(world_rectangle.corner(Corner::BottomLeft), WorldCoords{x: x + 50.0, y: y - 100.0});
            assert_eq!(world_rectangle.corner(Corner::BottomRight), WorldCoords{x: x + 50.0, y: y + 100.0});
        }

        Ok(())
    }

    #[test]
    fn is_inside_and_contains_test() -> Result<(), Box<dyn Error>> {

        //at origin, no rotation
        {
            let mut world_rectangle = WorldRectangle::new(200.0, 100.0);
            world_rectangle.set_rotation(0.0);
            world_rectangle.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            let origin = WorldCoords { x: 0.0, y: 0.0 };
            let top_left_corner = WorldCoords { x: -100.0, y: 50.0 };
            let top_right_corner = WorldCoords { x: 100.0, y: 50.0 };
            let bottom_left_corner = WorldCoords { x: -100.0, y: -50.0 };
            let bottom_right_corner = WorldCoords { x: 100.0, y: -50.0 };

            let all_edges = [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right];

            for &edge in all_edges.iter() {

                assert!(world_rectangle.is_inside(origin, edge));
                assert!(world_rectangle.is_inside(top_left_corner, edge));
                assert!(world_rectangle.is_inside(top_right_corner, edge));
                assert!(world_rectangle.is_inside(bottom_left_corner, edge));
                assert!(world_rectangle.is_inside(bottom_right_corner, edge));
            }

            let beyond_top_left_corner = top_left_corner + top_left_corner;
            let beyond_top_right_corner = top_right_corner + top_right_corner;
            let beyond_bottom_left_corner = bottom_left_corner + bottom_left_corner;
            let beyond_bottom_right_corner = bottom_right_corner + bottom_right_corner;

            assert!( ! world_rectangle.is_inside(beyond_top_left_corner, Edge::Top));
            assert!( ! world_rectangle.is_inside(beyond_top_right_corner, Edge::Top));
            assert!(   world_rectangle.is_inside(beyond_bottom_left_corner, Edge::Top));
            assert!(   world_rectangle.is_inside(beyond_bottom_right_corner, Edge::Top));

            assert!( ! world_rectangle.is_inside(beyond_top_left_corner, Edge::Left));
            assert!(   world_rectangle.is_inside(beyond_top_right_corner, Edge::Left));
            assert!( ! world_rectangle.is_inside(beyond_bottom_left_corner, Edge::Left));
            assert!(   world_rectangle.is_inside(beyond_bottom_right_corner, Edge::Left));

            assert!(   world_rectangle.is_inside(beyond_top_left_corner, Edge::Right));
            assert!( ! world_rectangle.is_inside(beyond_top_right_corner, Edge::Right));
            assert!(   world_rectangle.is_inside(beyond_bottom_left_corner, Edge::Right));
            assert!( ! world_rectangle.is_inside(beyond_bottom_right_corner, Edge::Right));

            assert!(   world_rectangle.is_inside(beyond_top_left_corner, Edge::Bottom));
            assert!(   world_rectangle.is_inside(beyond_top_right_corner, Edge::Bottom));
            assert!( ! world_rectangle.is_inside(beyond_bottom_left_corner, Edge::Bottom));
            assert!( ! world_rectangle.is_inside(beyond_bottom_right_corner, Edge::Bottom));


            assert!(world_rectangle.contains(origin));
            assert!(world_rectangle.contains(top_left_corner));
            assert!(world_rectangle.contains(top_right_corner));
            assert!(world_rectangle.contains(bottom_left_corner));
            assert!(world_rectangle.contains(bottom_right_corner));
            assert!( ! world_rectangle.contains(beyond_top_left_corner));
            assert!( ! world_rectangle.contains(beyond_top_right_corner));
            assert!( ! world_rectangle.contains(beyond_bottom_left_corner));
            assert!( ! world_rectangle.contains(beyond_bottom_right_corner));

        }

        Ok(())
    }

    #[test]
    fn convert_local_to_world_test() -> Result<(), Box<dyn Error>> {

        //at origin, no rotation
        {
            let mut world_rectangle = WorldRectangle::new(200.0, 100.0);
            world_rectangle.set_rotation(0.0);
            world_rectangle.set_translation(WorldCoords { x: 0.0, y: 0.0 });

            //top left corner
            {
                let pixel_coords = Vec3::new(0.0, 0.0, 0.0);
                let world_coords = Vec4::new(-100.0, 50.0, 0.0, 1.0);
                let m = world_rectangle.convert_local_to_world(pixel_coords);

                assert_eq!(m * Vec4::unit_w(), world_coords);
            }

            //bottom right corner
            {
                let pixel_coords = Vec3::new(200.0, 100.0, 0.0);
                let world_coords = Vec4::new(100.0, -50.0, 0.0, 1.0);
                let m = world_rectangle.convert_local_to_world(pixel_coords);

                assert_eq!(m * Vec4::unit_w(), world_coords);
            }
        }

        //rotated + translated
        {
            let mut world_rectangle = WorldRectangle::new(200.0, 100.0);
            world_rectangle.set_rotation(90.0);
            let x = 2000.0;
            let y = 1000.0;
            world_rectangle.set_translation(WorldCoords {x,y});

            //top left corner
            {
                let pixel_coords = Vec3::new(0.0, 0.0, 0.0);
                let world_coords = Vec4::new(x as f32 - 50.0, y as f32 - 100.0, 0.0, 1.0);
                let m = world_rectangle.convert_local_to_world(pixel_coords);

                assert_eq!(m * Vec4::unit_w(), world_coords);
            }

            //bottom right corner
            {
                let pixel_coords = Vec3::new(200.0, 100.0, 0.0);
                let world_coords = Vec4::new(x as f32 + 50.0, y as f32 + 100.0, 0.0, 1.0);
                let m = world_rectangle.convert_local_to_world(pixel_coords);

                assert_eq!(m * Vec4::unit_w(), world_coords);
            }
        }

        Ok(())
    }
}