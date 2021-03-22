/*
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


                    for m in &photos {

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


                    let points = &image0_control_points;

                    for &v in points {
                        let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                        let t1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)).concat(&t1);

                        let t1 = convert_photo_px_to_world(v, &photos[0]).concat(&t1);


                        color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.5, 0.2, 0.5)).unwrap();
                        color_mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
                    }

                    let points = &image1_control_points;

                    for &v in points {
                        let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                        let t1 = Mat4::from_angle_z(cgmath::Deg(45.0)).concat(&t1);
                        let t1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)).concat(&t1);

                        let t1 = convert_photo_px_to_world(v, &photos[1]).concat(&t1);

                        color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.8, 0.2, 0.5)).unwrap();
                        color_mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
                    }

                    if let Some(ref rp) = control_state.active_rotation_point {
                        let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                        let t1 = Mat4::from_angle_z(cgmath::Deg(-45.0)).concat(&t1);
                        let t1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)).concat(&t1);

                        let t1 = Mat4::from_translation(Vec3::new(rp.point.x as f32, rp.point.y as f32, 0.0)).concat(&t1);

                        color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 0.5)).unwrap();
                        color_mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
                    }

                    if let Some(ref rp) = control_state.active_rotation_point {
                        if let Some(ref rd) = control_state.active_rotate_drag {

                            //draw triangle to indicate dragged rotation angle

                            let mut cpu_mesh = CPUMesh {
                                positions: vec![
                                    rp.point.x as f32, rp.point.y as f32, 0.0,
                                    rd.mouse_start.x as f32, rd.mouse_start.y as f32, 0.0,
                                    rd.mouse_coords.x as f32, rd.mouse_coords.y as f32, 0.0,
                                ],

                                ..Default::default()
                            };

                            let mesh = Mesh::new(&context, &cpu_mesh).unwrap();

                            let t1 = Mat4::identity();

                            color_program.use_uniform_vec4("color", &Vec4::new(0.2, 0.2, 0.8, 0.5)).unwrap();
                            mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
                        }
                    }

                    gui.render().unwrap();

                    Ok(())
                }).unwrap();
*/