use three_d::definition::cpu_mesh::CPUMesh;
use three_d::object::{Mesh, MeshProgram};
use three_d::core::render_states::{CullType, BlendMultiplierType, BlendParameters, WriteMask, DepthTestType, RenderStates};
use three_d::core::render_target::{Screen, ClearState};
use three_d::math::{Vec2, Vec3, Vec4, Mat4};
use three_d::gui::GUI;
use three_d::{Transform,Context,CameraControl,FrameInput,SquareMatrix,InnerSpace};

use crate::control_state::{ControlState, DewarpShader};
use crate::photo::{Photo, Corner};
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
    Screen::write(&context, &ClearState::color_and_depth(0.2, 0.2, 0.2, 1.0, 1.0), || {
        let render_states = RenderStates {
            cull: CullType::None,

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


        for m in &entities.photos {
            let program = match control_state.dewarp_shader
            {
                DewarpShader::NoMorph => &texture_program,
                DewarpShader::Dewarp1 => &texture_dewarp_program,
                DewarpShader::Dewarp2 => &texture_dewarp2_program,
            };

            program.use_texture(&m.loaded_image_mesh.texture_2d, "tex").unwrap();

            m.loaded_image_mesh.mesh.render(program, render_states,
                                            frame_input.viewport, &m.to_world(), &camera)?;
        }


        if control_state.control_points_visible {

            let points = &entities.image0_control_points;

            for &v in points {
                let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                let t1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)).concat(&t1);

                let t1 = entities.photos[0].convert_photo_px_to_world(v).concat(&t1);


                color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.5, 0.2, 0.5)).unwrap();
                entities.color_mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
            }

            let points = &entities.image1_control_points;

            for &v in points {
                let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                let t1 = Mat4::from_angle_z(cgmath::Deg(45.0)).concat(&t1);
                let t1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)).concat(&t1);

                let t1 = entities.photos[1].convert_photo_px_to_world(v).concat(&t1);

                color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.8, 0.2, 0.5)).unwrap();
                entities.color_mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
            }
        }

        if let Some(ref rp) = control_state.active_rotation_point {
            let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
            let t1 = Mat4::from_angle_z(cgmath::Deg(-45.0)).concat(&t1);
            let t1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)).concat(&t1);

            let t1 = Mat4::from_translation(Vec3::new(rp.point.x as f32, rp.point.y as f32, 0.0)).concat(&t1);

            color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 0.5)).unwrap();
            entities.color_mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
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

                let mesh = Mesh::new(&context, &cpu_mesh).unwrap();

                let t1 = Mat4::identity();

                color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.2, 0.8, 0.5)).unwrap();
                mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;


                //draw angle lines to indicate dragged rotation angle
                let start_line_mat4 =   line_transform(&viewport_geometry, rp.point, mouse_start_resized,  1.0);
                let dragged_line_mat4 = line_transform(&viewport_geometry, rp.point, rd.mouse_coords, 1.0);

                color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 1.0)).unwrap();
                entities.line_mesh.render(&color_program, render_states, frame_input.viewport, &start_line_mat4,   &camera)?;
                entities.line_mesh.render(&color_program, render_states, frame_input.viewport, &dragged_line_mat4, &camera)?;

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
                entities.line_mesh.render(&color_program, render_states, frame_input.viewport, line, &camera)?;
            }
        }

        gui.render().unwrap();

        Ok(())
    }).unwrap();
}