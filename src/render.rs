use three_d::definition::CPUMesh;
use three_d::object::{Mesh, MeshProgram};
use three_d::core::{CullType, BlendMultiplierType, BlendParameters, BlendEquationType, WriteMask, DepthTestType, RenderStates};
use three_d::core::{Screen, ClearState};
use three_d::math::{Vec3, Vec4, Mat4};
use three_d::gui::GUI;
use three_d::{Transform, Context, CameraControl, FrameInput, SquareMatrix, ColorTargetTexture2D, Camera};

use crate::control_state::ControlState;
use crate::photo::Corner;
use crate::entities::Entities;
use crate::ViewportGeometry;

mod photo;
mod markers;
mod primitives;


/// Stores immutable references used in rendering
#[derive(Copy, Clone)]
pub struct Renderer<'a> {

    //three-d objects
    context: &'a Context,
    frame_input: &'a FrameInput,
    camera: &'a CameraControl,

    texture_program: &'a MeshProgram,
    texture_dewarp_program: &'a MeshProgram,
    texture_dewarp2_program: &'a MeshProgram,
    color_program: &'a MeshProgram,


    //crate objects
    viewport_geometry: &'a ViewportGeometry,
    control_state: &'a ControlState,
    entities: &'a Entities,
}


impl Renderer<'_> {

    pub fn new<'a>(
        context: &'a Context,
        frame_input: &'a FrameInput,
        camera: &'a CameraControl,

        texture_program: &'a MeshProgram,
        texture_dewarp_program: &'a MeshProgram,
        texture_dewarp2_program: &'a MeshProgram,
        color_program: &'a MeshProgram,

        viewport_geometry: &'a ViewportGeometry,
        control_state: &'a ControlState,
        entities: &'a Entities,
        ) -> Renderer<'a>
    {

        Renderer {
            context,
            frame_input,
            camera,

            texture_program,
            texture_dewarp_program,
            texture_dewarp2_program,
            color_program,

            viewport_geometry,
            control_state,
            entities,
        }
    }

    pub fn render(&self, gui: &mut GUI)
    {
        Screen::write(&self.context, ClearState::color_and_depth(0.2, 0.2, 0.2, 1.0, 1.0), || {

            if self.control_state.alignment_mode {
                //in alignment mode, use standard transparency
                self.render_photos(0.5, Renderer::render_states_transparency())?;
            }
            else {
                //in browse mode, use multipass rendering
                self.render_photos_with_pixel_averaging()?;
            }

            if self.control_state.control_points_visible {

                self.render_control_points_temp()?;
            }

            if let Some(ref rp) = self.control_state.active_rotation_point {

                self.draw_point(rp.point, -45.0, Vec4::new(0.8, 0.8, 0.2, 0.5))?;
            }

            if let Some(ref rp) = self.control_state.active_rotation_point {
                if let Some(ref rd) = self.control_state.active_rotate_drag {

                    self.draw_active_rotate_drag(rp, rd)?;
                }
            }

            //selected photo border rectangle
            if let Some(index) = self.control_state.selected_photo_index {

                self.draw_selected_photo_border_rectangle(index)?;
            }


            //render map overlay
            self.render_map_overlay();


            gui.render().unwrap();

            Ok(())
        }).unwrap();
    }


    fn render_map_overlay(&self) {

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

        overlay_texture.write(ClearState::color(0.0, 0.5, 0.0, 0.0), || {

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

    fn render_states_transparency() -> RenderStates {

        RenderStates {

            blend: Some(BlendParameters {
                source_rgb_multiplier: BlendMultiplierType::SrcAlpha,
                source_alpha_multiplier: BlendMultiplierType::One,
                destination_rgb_multiplier: BlendMultiplierType::OneMinusSrcAlpha,
                destination_alpha_multiplier: BlendMultiplierType::Zero,
                rgb_equation: BlendEquationType::Add,
                alpha_equation: BlendEquationType::Add,
            }),

            write_mask: WriteMask::COLOR,
            depth_test: DepthTestType::Always,
        }
    }

    fn render_states_accumulate() -> RenderStates {

        RenderStates {

            blend: Some(BlendParameters {
                source_rgb_multiplier: BlendMultiplierType::One,
                source_alpha_multiplier: BlendMultiplierType::One,
                destination_rgb_multiplier: BlendMultiplierType::One,
                destination_alpha_multiplier: BlendMultiplierType::One,
                rgb_equation: BlendEquationType::Add,
                alpha_equation: BlendEquationType::Add,
            }),

            write_mask: WriteMask::COLOR,
            depth_test: DepthTestType::Always,
        }
    }

    fn render_states_no_blend() -> RenderStates {

        RenderStates {
            blend: None,
            write_mask: WriteMask::COLOR,
            depth_test: DepthTestType::Always,
        }
    }
}