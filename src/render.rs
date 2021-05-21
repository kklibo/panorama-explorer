use three_d::definition::CPUMesh;
use three_d::object::{Mesh, MeshProgram};
use three_d::core::{CullType, BlendMultiplierType, BlendParameters, WriteMask, DepthTestType, RenderStates};
use three_d::core::{Screen, ClearState};
use three_d::math::{Vec2, Vec3, Vec4, Mat4};
use three_d::gui::GUI;
use three_d::{Transform, Context, CameraControl, FrameInput, SquareMatrix, InnerSpace, ColorTargetTexture2D, Camera};

use crate::control_state::{ControlState, DewarpShader};
use crate::photo::Corner;
use crate::entities::Entities;
use crate::viewport_geometry::{WorldCoords, ViewportGeometry};


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
            let render_states = RenderStates {

                blend: Some(BlendParameters {
                    source_rgb_multiplier: BlendMultiplierType::SrcAlpha,
                    source_alpha_multiplier: BlendMultiplierType::One,
                    destination_rgb_multiplier: BlendMultiplierType::OneMinusSrcAlpha,
                    destination_alpha_multiplier: BlendMultiplierType::Zero,
                    ..Default::default()
                }),

                write_mask: WriteMask::COLOR,
                depth_test: DepthTestType::Always,

                ..Default::default()
            };

            if self.control_state.alignment_mode {
                //in alignment mode, use standard transparency

                for m in &self.entities.photos {
                    let program = match self.control_state.dewarp_shader
                    {
                        DewarpShader::NoMorph => &self.texture_program,
                        DewarpShader::Dewarp1 => &self.texture_dewarp_program,
                        DewarpShader::Dewarp2 => &self.texture_dewarp2_program,
                    };

                    program.use_texture(&m.loaded_image_mesh.texture_2d, "tex").unwrap();
                    program.use_uniform_float("out_alpha", &0.5).unwrap();

                    let mut mesh = m.loaded_image_mesh.mesh.clone();
                    mesh.transformation = m.to_world();
                    mesh.render(program, render_states,
                                                self.frame_input.viewport, &self.camera)?;
                }
            }
            else {
                //in browse mode, use multipass rendering

                let photo_texture =
                Renderer::render_photos_to_texture(
                    &self.context,
                    &self.frame_input,
                    &self.control_state,
                    &self.camera,
                    &self.texture_program,
                    &self.texture_dewarp_program,
                    &self.texture_dewarp2_program,
                    &self.entities,
                );

                Screen::write(&self.context, ClearState::none(), || {

                    self.entities.copy_photos_effect.use_texture(&photo_texture, "colorMap")?;
                    self.entities.copy_photos_effect.apply(render_states, self.frame_input.viewport)

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
                    mesh.render(&self.color_program, render_states, self.frame_input.viewport, &self.camera)?;
                }

                let points = &self.entities.image1_control_points;

                for &v in points {
                    let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                    let t1 = Mat4::from_angle_z(cgmath::Deg(45.0)).concat(&t1);

                    let t1 = self.entities.photos[1].convert_photo_px_to_world(v).concat(&t1);

                    self.color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.8, 0.2, 0.5)).unwrap();
                    let mut mesh = self.entities.color_mesh.clone();
                    mesh.transformation = t1;
                    mesh.render(&self.color_program, render_states, self.frame_input.viewport, &self.camera)?;
                }
            }

            if let Some(ref rp) = self.control_state.active_rotation_point {
                let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                let t1 = Mat4::from_angle_z(cgmath::Deg(-45.0)).concat(&t1);

                let t1 = Mat4::from_translation(Vec3::new(rp.point.x as f32, rp.point.y as f32, 0.0)).concat(&t1);

                self.color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 0.5)).unwrap();
                let mut mesh = self.entities.color_mesh.clone();
                mesh.transformation = t1;
                mesh.render(&self.color_program, render_states, self.frame_input.viewport, &self.camera)?;
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
                    mesh.render(&self.color_program, render_states, self.frame_input.viewport, &self.camera)?;


                    //draw angle lines to indicate dragged rotation angle
                    let start_line_mat4 =   line_transform(&self.viewport_geometry, rp.point, mouse_start_resized,  1.0);
                    let dragged_line_mat4 = line_transform(&self.viewport_geometry, rp.point, rd.mouse_coords, 1.0);

                    self.color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 1.0)).unwrap();
                    let mut mesh =  self.entities.line_mesh.clone();
                    mesh.transformation = start_line_mat4;
                    mesh.render(&self.color_program, render_states, self.frame_input.viewport, &self.camera)?;
                    mesh.transformation = dragged_line_mat4;
                    mesh.render(&self.color_program, render_states, self.frame_input.viewport, &self.camera)?;

                }
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

                let t1 = Mat4::from_nonuniform_scale(
                    line_x.magnitude(),
                    pixel_thickness * viewport_geometry.world_units_per_pixel() as f32,
                    1.0
                );
                let t1 = Mat4::from_angle_z(angle).concat(&t1);
                let t1 = Mat4::from_translation(p1v).concat(&t1);

                t1
            }

            //selected photo border rectangle
            if let Some(index) = self.control_state.selected_photo_index {

                let mut lines = Vec::<Mat4>::new();

                let mut add_corner_line = |corner1: Corner, corner2: Corner| {
                    lines.push(line_transform(
                        &self.viewport_geometry,
                        self.entities.photos[index].corner(corner1),
                        self.entities.photos[index].corner(corner2),
                        1.0
                    ));
                };

                add_corner_line(Corner::BottomLeft, Corner::BottomRight);
                add_corner_line(Corner::BottomRight, Corner::TopRight);
                add_corner_line(Corner::TopRight, Corner::TopLeft);
                add_corner_line(Corner::TopLeft, Corner::BottomLeft);

                //draw lines
                for ref line in lines {
                    self.color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.8, 0.2, 1.0)).unwrap();
                    let mut mesh  = self.entities.line_mesh.clone();
                    mesh.transformation = *line;
                    mesh.render(&self.color_program, render_states, self.frame_input.viewport, &self.camera)?;
                }
            }


            //render map overlay
            Renderer::render_map_overlay(
                self.context,
                self.frame_input,
                self.viewport_geometry,
                self.texture_program,
                render_states,
                self.entities,
            );


            gui.render().unwrap();

            Ok(())
        }).unwrap();
    }


    fn render_map_overlay(
        context: &Context,
        frame_input: &FrameInput,
        viewport_geometry: &ViewportGeometry,
        texture_program: &MeshProgram,
        render_states: RenderStates,
        entities: &Entities,
    )
    {

        //create texture for overlay contents
        use three_d::definition::{Interpolation, Wrapping, Format};
        use three_d::vec3;
        use three_d::Viewport;

        let overlay_width_px: u32 = 300;
        let overlay_height_px: u32 = 300;

        let overlay_texture = ColorTargetTexture2D::<u8>::new(
            &context,
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

            texture_program.use_texture(&entities.overlay_mesh.texture_2d, "tex").unwrap();
            texture_program.use_uniform_float("out_alpha", &1.0).unwrap();

            let viewport = Viewport::new_at_origo(overlay_width_px,overlay_height_px);
            let camera = Camera::new_orthographic(&context,
                                         vec3(0.0, 0.0, 5.0),
                                         vec3(0.0, 0.0, 0.0),
                                         vec3(0.0, 1.0, 0.0),
                                         1.0,
                                         1.0,
                                         10.0).unwrap();

            let mut mesh = entities.overlay_mesh.mesh.clone();
            mesh.transformation = Mat4::identity()
                //flip y-coords
                .concat(&Mat4::from_nonuniform_scale(1.0, -1.0, 1.0)
                .concat(&Mat4::from_scale(2.0))
                );

            mesh.cull = CullType::None;
            mesh.render(texture_program, render_states, viewport, &camera)?;

            Ok(())

        }).unwrap();


        //three-d infrastructure for overlay
        let cpu_mesh = CPUMesh {
            positions: crate::entities::square_positions(),
            uvs: Some(crate::entities::square_uvs()),

            ..Default::default()
        };

        let mut mesh = Mesh::new(&context, &cpu_mesh).unwrap();
        mesh.cull = CullType::Back;

        let render_states = RenderStates {
            write_mask: WriteMask::COLOR,
            depth_test: DepthTestType::Always,

            ..Default::default()
        };



        let viewport_width = viewport_geometry.width_in_pixels().get() as f32;
        let viewport_height = viewport_geometry.height_in_pixels().get() as f32;

        //orthographic camera view for UI rendering in viewport
        let camera = Camera::new_orthographic(&context,
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

        mesh.render_with_texture(&overlay_texture, render_states, frame_input.viewport, &camera).unwrap();

    }


    pub fn render_photos_to_texture(
        context: &Context,
        frame_input: &FrameInput,
        control_state: &ControlState,
        camera: &CameraControl,
        texture_program: &MeshProgram,
        texture_dewarp_program: &MeshProgram,
        texture_dewarp2_program: &MeshProgram,
        entities: &Entities
    ) -> ColorTargetTexture2D<u8>
    {

        use three_d::definition::{Interpolation, Wrapping, Format};

        let tmp_texture = ColorTargetTexture2D::<f32>::new(
            &context,
            frame_input.viewport.width,
            frame_input.viewport.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Format::RGBA,
        ).unwrap();

        let out_texture = ColorTargetTexture2D::<u8>::new(
            &context,
            frame_input.viewport.width,
            frame_input.viewport.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Format::RGBA,
        ).unwrap();

        {
            tmp_texture.write(ClearState::color(0.0, 0.0, 0.0, 0.0), || {
                let render_states = RenderStates {

                    blend: Some(BlendParameters {
                        source_rgb_multiplier: BlendMultiplierType::One,
                        source_alpha_multiplier: BlendMultiplierType::One,
                        destination_rgb_multiplier: BlendMultiplierType::One,
                        destination_alpha_multiplier: BlendMultiplierType::One,
                        ..Default::default()
                    }),

                    write_mask: WriteMask::COLOR,
                    depth_test: DepthTestType::Always,

                    ..Default::default()
                };


                for m in &entities.photos {
                    let program = match control_state.dewarp_shader
                    {
                        DewarpShader::NoMorph => &texture_program,
                        DewarpShader::Dewarp1 => &texture_dewarp_program,
                        DewarpShader::Dewarp2 => &texture_dewarp2_program,
                    };

                    program.use_texture(&m.loaded_image_mesh.texture_2d, "tex").unwrap();
                    program.use_uniform_float("out_alpha", &1.0).unwrap();

                    let mut mesh =  m.loaded_image_mesh.mesh.clone();
                    mesh.transformation = m.to_world();
                    mesh.render(program, render_states,
                                                    frame_input.viewport, &camera)?;
                }

                Ok(())
            }).unwrap();

            let render_states =
                RenderStates {
                    depth_test: DepthTestType::Always,
                    write_mask: WriteMask::COLOR,
                    ..Default::default()
                };

            out_texture.write(ClearState::none(), || {
                entities.average_effect.use_texture(&tmp_texture, "colorMap")?;
                entities.average_effect.apply(render_states, frame_input.viewport)

            }).unwrap();
        }

        out_texture
    }
}