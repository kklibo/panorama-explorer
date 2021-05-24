use three_d::definition::CPUMesh;
use three_d::object::{Mesh, MeshProgram};
use three_d::core::{CullType, BlendMultiplierType, BlendParameters, BlendEquationType, WriteMask, DepthTestType, RenderStates};
use three_d::core::{Screen, ClearState};
use three_d::math::{Vec2, Vec3, Vec4, Mat4};
use three_d::gui::GUI;
use three_d::{Transform, Context, CameraControl, FrameInput, SquareMatrix, InnerSpace, ColorTargetTexture2D, Camera};
use three_d::Error;

use crate::control_state::ControlState;
use crate::photo::Corner;
use crate::entities::Entities;
use crate::viewport_geometry::{WorldCoords, ViewportGeometry};

mod photo;


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
                self.render_photos()?;
            }
            else {
                //in browse mode, use multipass rendering

                let photo_texture = self.render_photos_to_texture();

                Screen::write(&self.context, ClearState::none(), || {

                    self.entities.copy_photos_effect.use_texture(&photo_texture, "colorMap")?;
                    self.entities.copy_photos_effect.apply(Renderer::render_states_transparency(), self.frame_input.viewport)

                }).unwrap();
            }

            if self.control_state.control_points_visible {

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
            }

            if let Some(ref rp) = self.control_state.active_rotation_point {

                self.draw_point(rp.point, -45.0, Vec4::new(0.8, 0.8, 0.2, 0.5))?;
            }

            if let Some(ref rp) = self.control_state.active_rotation_point {
                if let Some(ref rd) = self.control_state.active_rotate_drag {

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
                    self.draw_line(rp.point, rd.mouse_coords,  1.0, line_color)?;

                }
            }

            //selected photo border rectangle
            if let Some(index) = self.control_state.selected_photo_index {

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
                draw_corner_line(Corner::TopLeft, Corner::BottomLeft)?;
            }


            //render map overlay
            self.render_map_overlay();


            gui.render().unwrap();

            Ok(())
        }).unwrap();
    }


    fn draw_point(&self, point: WorldCoords, rotation_deg: f32, color: Vec4) -> Result<(), Error> {

        let mut mesh = self.entities.color_mesh.clone();

        let translate_to_point = Mat4::from_translation(Vec3::new(point.x as f32, point.y as f32, 0.0));
        let rotate_marker = Mat4::from_angle_z(cgmath::Deg(rotation_deg));
        let scale_marker = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);

        mesh.transformation = translate_to_point * rotate_marker * scale_marker;

        self.color_program.use_uniform_vec4("color", &color).unwrap();
        mesh.render(&self.color_program, Renderer::render_states_transparency(), self.frame_input.viewport, &self.camera)
    }

    fn draw_line(
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