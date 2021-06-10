use three_d::{InnerSpace,SquareMatrix,Vec2,Vec4,Mat4};
use three_d::{CPUMesh,Mesh,CullType};
use three_d::Error;

use crate::WorldCoords;
use crate::control_state::{RotationPoint,RotateDrag};

use super::{Renderer,colors,render_states};
use crate::photo::Photo;
use crate::world_rectangle::Corner;
use crate::viewport_geometry::PixelCoords;

impl Renderer<'_> {

    pub(in super) fn render_control_points_temp(&self) -> Result<(), Error> {

        let points = &self.entities.image0_control_points;

        for &v in points {

            let pixel_coords = PixelCoords{ x: v.x as f64, y: v.y as f64};
            let world_coords = self.entities.photos[0].world_coords(pixel_coords);

            self.draw_point(world_coords, 0.0, colors::photo1_control_points_temp())?;
        }

        let points = &self.entities.image1_control_points;

        for &v in points {

            let pixel_coords = PixelCoords{ x: v.x as f64, y: v.y as f64};
            let world_coords = self.entities.photos[1].world_coords(pixel_coords);

            self.draw_point(world_coords, 45.0, colors::photo2_control_points_temp())?;
        }

        Ok(())
    }

    pub(in super) fn draw_active_rotate_drag(&self, rp: &RotationPoint, rd: &RotateDrag) -> Result<(), Error> {

        //create resized line segment for dragged rotation start line
        let mouse_coords_vec_length =
            Vec2::new(
                rd.mouse_coords.x as f32 - rp.point.x as f32,
                rd.mouse_coords.y as f32 - rp.point.y as f32,
            ).magnitude();

        let mouse_start_vec2 =
            Vec2::new(
                rd.mouse_start.x as f32 - rp.point.x as f32,
                rd.mouse_start.y as f32 - rp.point.y as f32,
            )
            .normalize() * mouse_coords_vec_length
            + Vec2::new(rp.point.x as f32, rp.point.y as f32);

        let mouse_start_resized = WorldCoords{x: mouse_start_vec2.x as f64, y: mouse_start_vec2.y as f64};


        //draw triangle to indicate dragged rotation angle
        let cpu_mesh = CPUMesh {
            positions: vec![
                rp.point.x as f32, rp.point.y as f32, 0.0,
                mouse_start_resized.x as f32, mouse_start_resized.y as f32, 0.0,
                rd.mouse_coords.x as f32, rd.mouse_coords.y as f32, 0.0,
            ],

            ..Default::default()
        };

        let mut mesh = Mesh::new(&self.context, &cpu_mesh).unwrap();
        mesh.cull = CullType::None;
        let t1 = Mat4::identity();

        self.color_program.use_uniform_vec4("color", &colors::dragged_rotation_triangle()).unwrap();
        mesh.transformation = t1;
        mesh.render(&self.color_program, render_states::render_states_transparency(), self.frame_input.viewport, &self.camera)?;


        //draw angle lines to indicate dragged rotation angle
        let line_color = colors::dragged_rotation_angle_lines();
        self.draw_line(rp.point, mouse_start_resized,  1.0, line_color)?;
        self.draw_line(rp.point, rd.mouse_coords,  1.0, line_color)
    }

    fn draw_photo_border_rectangle(&self, photo: &Photo, color: Vec4) -> Result<(), Error> {

        let draw_corner_line = |corner1: Corner, corner2: Corner| {

            self.draw_line(
                photo.orientation().corner(corner1),
                photo.orientation().corner(corner2),
                1.0,
                color,
            )
        };

        draw_corner_line(Corner::BottomLeft, Corner::BottomRight)?;
        draw_corner_line(Corner::BottomRight, Corner::TopRight)?;
        draw_corner_line(Corner::TopRight, Corner::TopLeft)?;
        draw_corner_line(Corner::TopLeft, Corner::BottomLeft)
    }

    pub(in super) fn draw_selected_photo_border_rectangle(&self, photo: &Photo) -> Result<(), Error> {

        self.draw_photo_border_rectangle(photo, colors::selected_photo_border_rectangle())
    }

    pub(in super) fn draw_photo_border_rectangles(&self, photos: &Vec<Photo>) -> Result<(), Error> {

        for photo in photos {

            self.draw_photo_border_rectangle(photo, colors::photo_border_rectangle())?;
        }

        Ok(())
    }
}