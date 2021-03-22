/*
            let mut panel_width = frame_input.viewport.width / 10;
            redraw |= gui.update(&mut frame_input, |gui_context| {

                use three_d::egui_gui::egui::{SidePanel, Slider};
                SidePanel::left("side_panel", panel_width as f32).show(gui_context, |ui| {
                    ui.heading("panorama_tool");
                    ui.separator();

                    ui.heading("Left-click Tool:");
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::RotationPoint, format!("{:?}", MouseTool::RotationPoint));
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::DragToRotate,  format!("{:?}", MouseTool::DragToRotate ));
                    ui.separator();

                    ui.heading("Lens Correction");

                    let slider = Slider::f32(&mut control_state.dewarp_strength, 0.0..=10.0)
                        .text("dewarp strength")
                        .clamp_to_range(true);

                    if ui.add(slider).changed() {
                        update_shader_uniforms(&control_state.dewarp_strength);
                    }
                    ui.separator();

                    ui.heading("rotation test");
                    let slider = Slider::f32(&mut control_state.debug_rotation, -1.0..=1.0)
                        .text("angle")
                        .clamp_to_range(true);
                    if ui.add(slider).changed() {

                        if let Some(ref rp) = control_state.active_rotation_point {
                            //reset to values from start of rotation before rotate_around_point
                            photos[1].set_rotation(rp.rotate_start);
                            photos[1].set_translation(rp.translate_start);
                            photos[1].rotate_around_point(control_state.debug_rotation, rp.point);
                        }
                    }
                    ui.separator();

                    ui.heading("Dewarp Shader");
                    ui.radio_value(&mut control_state.dewarp_shader, DewarpShader::NoMorph, format!("{:?}", DewarpShader::NoMorph));
                    ui.radio_value(&mut control_state.dewarp_shader, DewarpShader::Dewarp1, format!("{:?}", DewarpShader::Dewarp1));
                    ui.radio_value(&mut control_state.dewarp_shader, DewarpShader::Dewarp2, format!("{:?}", DewarpShader::Dewarp2));
                    ui.separator();

                    ui.heading("Mouse Info");
                    ui.label(&control_state.mouse_click_ui_text);
                    ui.separator();

                    ui.heading("Photo Info");
                    ui.label(&control_state.photo_ui_text);
                    ui.separator();


                });
                panel_width = (gui_context.used_size().x * gui_context.pixels_per_point()) as usize;
            }).unwrap();
*/