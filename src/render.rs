use three_d::definition::CPUMesh;
use three_d::object::{Mesh, MeshProgram};
use three_d::core::{CullType, BlendMultiplierType, BlendParameters, WriteMask, DepthTestType, RenderStates};
use three_d::core::{Screen, ClearState};
use three_d::math::{Vec2, Vec3, Vec4, Mat4};
use three_d::gui::GUI;
use three_d::{Transform,Context,CameraControl,FrameInput,SquareMatrix,InnerSpace,ColorTargetTexture2D};

use crate::control_state::{ControlState, DewarpShader};
use crate::photo::Corner;
use crate::entities::Entities;
use crate::viewport_geometry::{WorldCoords, ViewportGeometry};


pub fn render(
    context: &Context,
    frame_input: &FrameInput,
    gui: &mut GUI,
    control_state: &ControlState,
    camera: &CameraControl,
    viewport_geometry: &ViewportGeometry,
    texture_program: &MeshProgram,
    texture_dewarp_program: &MeshProgram,
    texture_dewarp2_program: &MeshProgram,
    color_program: &MeshProgram,
    entities: &Entities
) {
    Screen::write(&context, ClearState::color_and_depth(0.2, 0.2, 0.2, 1.0, 1.0), || {
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

        if control_state.alignment_mode {
            //in alignment mode, use standard transparency

            for m in &entities.photos {
                let program = match control_state.dewarp_shader
                {
                    DewarpShader::NoMorph => &texture_program,
                    DewarpShader::Dewarp1 => &texture_dewarp_program,
                    DewarpShader::Dewarp2 => &texture_dewarp2_program,
                };

                program.use_texture(&m.loaded_image_mesh.texture_2d, "tex").unwrap();
                program.use_uniform_float("out_alpha", &0.5).unwrap();

                let mut mesh = m.loaded_image_mesh.mesh.clone();
                mesh.transformation = m.to_world();
                mesh.render(program, render_states,
                                            frame_input.viewport, &camera)?;
            }
        }
        else {
            //in browse mode, use multipass rendering

            let photo_texture =
            render_photos_to_texture(
                &context,
                &frame_input,
                &control_state,
                &camera,
                &texture_program,
                &texture_dewarp_program,
                &texture_dewarp2_program,
                &entities,
            );

            Screen::write(&context, ClearState::none(), || {

                entities.copy_photos_effect.use_texture(&photo_texture, "colorMap")?;
                entities.copy_photos_effect.apply(render_states, frame_input.viewport)

            }).unwrap();
        }

        if control_state.control_points_visible {

            let points = &entities.image0_control_points;

            for &v in points {
                let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);

                let t1 = entities.photos[0].convert_photo_px_to_world(v).concat(&t1);


                color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.5, 0.2, 0.5)).unwrap();
                let mut mesh = entities.color_mesh.clone();
                mesh.transformation = t1;
                mesh.render(&color_program, render_states, frame_input.viewport, &camera)?;
            }

            let points = &entities.image1_control_points;

            for &v in points {
                let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                let t1 = Mat4::from_angle_z(cgmath::Deg(45.0)).concat(&t1);

                let t1 = entities.photos[1].convert_photo_px_to_world(v).concat(&t1);

                color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.8, 0.2, 0.5)).unwrap();
                let mut mesh = entities.color_mesh.clone();
                mesh.transformation = t1;
                mesh.render(&color_program, render_states, frame_input.viewport, &camera)?;
            }
        }

        if let Some(ref rp) = control_state.active_rotation_point {
            let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
            let t1 = Mat4::from_angle_z(cgmath::Deg(-45.0)).concat(&t1);

            let t1 = Mat4::from_translation(Vec3::new(rp.point.x as f32, rp.point.y as f32, 0.0)).concat(&t1);

            color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 0.5)).unwrap();
            let mut mesh = entities.color_mesh.clone();
            mesh.transformation = t1;
            mesh.render(&color_program, render_states, frame_input.viewport, &camera)?;
        }

        if let Some(ref rp) = control_state.active_rotation_point {
            if let Some(ref rd) = control_state.active_rotate_drag {

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

                let mut mesh = Mesh::new(&context, &cpu_mesh).unwrap();
                mesh.cull = CullType::None;
                let t1 = Mat4::identity();

                color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.2, 0.8, 0.5)).unwrap();
                mesh.transformation = t1;
                mesh.render(&color_program, render_states, frame_input.viewport, &camera)?;


                //draw angle lines to indicate dragged rotation angle
                let start_line_mat4 =   line_transform(&viewport_geometry, rp.point, mouse_start_resized,  1.0);
                let dragged_line_mat4 = line_transform(&viewport_geometry, rp.point, rd.mouse_coords, 1.0);

                color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 1.0)).unwrap();
                let mut mesh =  entities.line_mesh.clone();
                mesh.transformation = start_line_mat4;
                mesh.render(&color_program, render_states, frame_input.viewport, &camera)?;
                mesh.transformation = dragged_line_mat4;
                mesh.render(&color_program, render_states, frame_input.viewport, &camera)?;

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
        if let Some(index) = control_state.selected_photo_index {

            let mut lines = Vec::<Mat4>::new();

            let mut add_corner_line = |corner1: Corner, corner2: Corner| {
                lines.push(line_transform(
                    &viewport_geometry,
                    entities.photos[index].corner(corner1),
                    entities.photos[index].corner(corner2),
                    1.0
                ));
            };

            add_corner_line(Corner::BottomLeft, Corner::BottomRight);
            add_corner_line(Corner::BottomRight, Corner::TopRight);
            add_corner_line(Corner::TopRight, Corner::TopLeft);
            add_corner_line(Corner::TopLeft, Corner::BottomLeft);

            //draw lines
            for ref line in lines {
                color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.8, 0.2, 1.0)).unwrap();
                let mut mesh  = entities.line_mesh.clone();
                mesh.transformation = *line;
                mesh.render(&color_program, render_states, frame_input.viewport, &camera)?;
            }
        }

        gui.render().unwrap();

        Ok(())
    }).unwrap();
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
