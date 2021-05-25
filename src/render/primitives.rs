use three_d::{InnerSpace,Vec2,Vec3,Vec4,Mat4};
use three_d::Error;

use crate::{ViewportGeometry,WorldCoords};

use super::Renderer;

impl Renderer<'_> {

    pub(in super) fn draw_point(&self, point: WorldCoords, rotation_deg: f32, color: Vec4) -> Result<(), Error> {

        let mut mesh = self.entities.color_mesh.clone();

        let translate_to_point = Mat4::from_translation(Vec3::new(point.x as f32, point.y as f32, 0.0));
        let rotate_marker = Mat4::from_angle_z(cgmath::Deg(rotation_deg));
        let scale_marker = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);

        mesh.transformation = translate_to_point * rotate_marker * scale_marker;

        self.color_program.use_uniform_vec4("color", &color).unwrap();
        mesh.render(&self.color_program, Renderer::render_states_transparency(), self.frame_input.viewport, &self.camera)
    }

    pub(in super) fn draw_line(
        &self,
        point1: WorldCoords,
        point2: WorldCoords,
        pixel_thickness: f32,
        color: Vec4,
    ) -> Result<(), Error>
    {
        let mut mesh =  self.entities.line_mesh.clone();

        mesh.transformation = Renderer::line_transform(&self.viewport_geometry, point1, point2, pixel_thickness);

        self.color_program.use_uniform_vec4("color", &color).unwrap();
        mesh.render(&self.color_program, Renderer::render_states_transparency(), self.frame_input.viewport, &self.camera)
    }

    fn line_transform(
        viewport_geometry: &ViewportGeometry,
        p1: WorldCoords,
        p2: WorldCoords,
        pixel_thickness: f32,
    ) -> Mat4
    {
        let p1v = Vec3::new(p1.x as f32, p1.y as f32, 0.0);

        let dx = (p2.x - p1.x) as f32;
        let dy = (p2.y - p1.y) as f32;

        let line_x = Vec2::new(dx, dy);

        let angle = Vec2::unit_x().angle(line_x);

        let translate_to_p1 = Mat4::from_translation(p1v);
        let rotate_around_p1 = Mat4::from_angle_z(angle);
        let scale_length_and_thickness =
            Mat4::from_nonuniform_scale(
                line_x.magnitude(),
                pixel_thickness * viewport_geometry.world_units_per_pixel() as f32,
                1.0
            );

        translate_to_p1 * rotate_around_p1 * scale_length_and_thickness
    }
}