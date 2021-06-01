use three_d::{Mesh,CPUMesh,ColorTargetTexture2D};
use three_d::{Camera,CullType};
use three_d::{Vec3,Mat4,SquareMatrix,Transform};

use super::{Renderer,colors};

impl Renderer<'_> {


    pub(in super) fn render_map_overlay(&self) {

        //create texture for overlay contents
        use three_d::definition::{Interpolation, Wrapping, Format};
        use three_d::vec3;
        use three_d::Viewport;

        let overlay_width_px: u32 = 300;
        let overlay_height_px: u32 = 300;

        let overlay_texture = ColorTargetTexture2D::<u8>::new(
            &self.context,
            overlay_width_px,
            overlay_height_px,
            Interpolation::Linear,
            Interpolation::Linear,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Format::RGBA,
        ).unwrap();

        overlay_texture.write(colors::map_overlay_clear(), || {

            self.texture_program.use_texture(&self.entities.overlay_mesh.texture_2d, "tex").unwrap();
            self.texture_program.use_uniform_float("out_alpha", &1.0).unwrap();

            let viewport = Viewport::new_at_origo(overlay_width_px,overlay_height_px);
            let camera = Camera::new_orthographic(&self.context,
                                         vec3(0.0, 0.0, 5.0),
                                         vec3(0.0, 0.0, 0.0),
                                         vec3(0.0, 1.0, 0.0),
                                         1.0,
                                         1.0,
                                         10.0).unwrap();

            let mut mesh = self.entities.overlay_mesh.mesh.clone();
            mesh.transformation = Mat4::identity()
                //flip y-coords
                .concat(&Mat4::from_nonuniform_scale(1.0, -1.0, 1.0)
                .concat(&Mat4::from_scale(2.0))
                );

            mesh.cull = CullType::None;
            mesh.render(self.texture_program, Renderer::render_states_transparency(), viewport, &camera)?;

            Ok(())

        }).unwrap();


        //three-d infrastructure for overlay
        let cpu_mesh = CPUMesh {
            positions: crate::entities::square_positions(),
            uvs: Some(crate::entities::square_uvs()),

            ..Default::default()
        };

        let mut mesh = Mesh::new(&self.context, &cpu_mesh).unwrap();
        mesh.cull = CullType::Back;


        let viewport_width = self.viewport_geometry.width_in_pixels().get() as f32;
        let viewport_height = self.viewport_geometry.height_in_pixels().get() as f32;

        //orthographic camera view for UI rendering in viewport
        let camera = Camera::new_orthographic(&self.context,
                                     vec3(0.0, 0.0, 5.0),
                                     vec3(0.0, 0.0, 0.0),
                                     vec3(0.0, 1.0, 0.0),
                                     viewport_width,
                                     viewport_height,
                                     10.0).unwrap();


        //temp hardcode: render overlay in square near lower right corner
        let t1 = Mat4::from_scale(300.0);
        let t1 = Mat4::from_translation(Vec3::new(viewport_width*0.5 - 160.0, viewport_height*-0.5 + 160.0, 0.0)).concat(&t1);
        mesh.transformation = t1;

        mesh.render_with_texture(&overlay_texture, Renderer::render_states_no_blend(), self.frame_input.viewport, &camera).unwrap();

    }
}