use three_d::{Transform,InnerSpace,SquareMatrix,Vec2,Vec4,Mat4};
use three_d::{CPUMesh,Mesh,CullType};
use three_d::Error;

use crate::WorldCoords;
use crate::control_state::{RotationPoint,RotateDrag};

use super::{Renderer,Corner};

impl Renderer<'_> {

    pub(in super) fn render_control_points_temp(&self) -> Result<(), Error> {

        let points = &self.entities.image0_control_points;

        for &v in points {
            let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);

            let t1 = self.entities.photos[0].convert_photo_px_to_world(v).concat(&t1);


            self.color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.5, 0.2, 0.5)).unwrap();
            let mut mesh = self.entities.color_mesh.clone();
            mesh.transformation = t1;
            mesh.render(&self.color_program, Renderer::render_states_transparency(), self.frame_input.viewport, &self.camera)?;
        }

        let points = &self.entities.image1_control_points;

        for &v in points {
            let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
            let t1 = Mat4::from_angle_z(cgmath::Deg(45.0)).concat(&t1);

            let t1 = self.entities.photos[1].convert_photo_px_to_world(v).concat(&t1);

            self.color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.8, 0.2, 0.5)).unwrap();
            let mut mesh = self.entities.color_mesh.clone();
            mesh.transformation = t1;
            mesh.render(&self.color_program, Renderer::render_states_transparency(), self.frame_input.viewport, &self.camera)?;
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

        self.color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.2, 0.8, 0.5)).unwrap();
        mesh.transformation = t1;
        mesh.render(&self.color_program, Renderer::render_states_transparency(), self.frame_input.viewport, &self.camera)?;


        //draw angle lines to indicate dragged rotation angle
        let line_color = Vec4::new(0.8, 0.8, 0.2, 1.0);
        self.draw_line(rp.point, mouse_start_resized,  1.0, line_color)?;
        self.draw_line(rp.point, rd.mouse_coords,  1.0, line_color)
    }

    pub(in super) fn draw_selected_photo_border_rectangle(&self, index: usize) -> Result<(), Error> {

        let draw_corner_line = |corner1: Corner, corner2: Corner| {

            self.draw_line(
                self.entities.photos[index].corner(corner1),
                self.entities.photos[index].corner(corner2),
                1.0,
                Vec4::new(0.2, 0.8, 0.2, 1.0),
            )
        };

        draw_corner_line(Corner::BottomLeft, Corner::BottomRight)?;
        draw_corner_line(Corner::BottomRight, Corner::TopRight)?;
        draw_corner_line(Corner::TopRight, Corner::TopLeft)?;
        draw_corner_line(Corner::TopLeft, Corner::BottomLeft)
    }
}