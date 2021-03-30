use three_d::camera::CameraControl;
use three_d::frame::FrameInput;
use three_d::frame::input::{Event, MouseButton, State, Key};
use three_d::gui::GUI;
use three_d::egui_gui::egui::{SidePanel, Slider, Button};
use three_d::math::{Vec2, InnerSpace};

use log::info;

use crate::viewport_geometry::{ViewportGeometry, PixelCoords, WorldCoords};
use crate::control_state::{ControlState, MouseTool, DewarpShader, Pan, Drag, RotateDrag, RotationPoint};
use crate::photo::Photo;
use crate::entities::Entities;

pub fn run_gui_controls(frame_input: &mut FrameInput, gui: &mut GUI, control_state: &mut ControlState, entities: &mut Entities) -> bool {

    let mut panel_width = frame_input.viewport.width / 10;
    let redraw = gui.update(frame_input, |gui_context| {

        SidePanel::left("side_panel", panel_width as f32).show(gui_context, |ui| {
            ui.heading("panorama_tool");
            ui.separator();

            ui.heading("Left-click Tool:");
            ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::SelectPhoto, format!("{:?}", MouseTool::SelectPhoto));
            ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::RotationPoint, format!("{:?}", MouseTool::RotationPoint));
            ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::DragToRotate, format!("{:?}", MouseTool::DragToRotate));
            ui.separator();

            ui.heading("Dewarp Shader");
            ui.radio_value(&mut control_state.dewarp_shader, DewarpShader::NoMorph, format!("Off"));
            ui.radio_value(&mut control_state.dewarp_shader, DewarpShader::Dewarp2, format!("On"));
            ui.separator();

            ui.heading("Mouse Info");
            ui.label(&control_state.mouse_click_ui_text);
            ui.separator();

            ui.heading("Mouse Location");
            ui.label(&control_state.mouse_location_ui_text);
            ui.separator();

            ui.heading("Photo Info");
            ui.label(&control_state.photo_ui_text);
            ui.separator();

            if ui.add(Button::new("reset photos")).clicked() {

                entities.set_photos_from_json_serde_string(&entities.photo_persistent_settings_string.clone()).unwrap();
            }

            if ui.add(Button::new("dump debug info")).clicked() {
                for ph in &entities.photos {
                    info!("{}", serde_json::to_string(ph).unwrap());
                }
            }
        });
        panel_width = (gui_context.used_size().x * gui_context.pixels_per_point()) as usize;
    }).unwrap();

    redraw
}

pub fn handle_input_events(
    frame_input: &mut FrameInput,
    control_state: &mut ControlState,
    viewport_geometry: &mut ViewportGeometry,
    camera: &mut CameraControl,
    photos: &mut Vec<Photo>,
) -> bool {

    let mut redraw = false;

    for event in frame_input.events.iter() {
        match event {
            Event::MouseClick {state, button, position, handled, ..} => {
                info!("MouseClick: {:?}", event);
                control_state.mouse_click_ui_text = format!("MouseClick: {:#?}", event);

                if *handled {break};

                let world_coords =
                viewport_geometry.pixels_to_world(&PixelCoords{x: position.0, y: position.1});
                info!("  WorldCoords: {:?}", world_coords);


                match *button {

                    MouseButton::Middle => control_state.active_pan =
                        match *state {
                            State::Pressed => {
                                Some(Pan {
                                    mouse_start: *position,
                                    camera_start: camera.position().clone(),
                                })
                            },
                            State::Released => None,
                        },

                    MouseButton::Right =>
                        match *state {
                            State::Pressed => {

                                control_state.active_drag = None;

                                for (i, ph) in photos.iter().enumerate() {
                                    if ph.contains(world_coords) {
                                        info!("clicked on photos[{}]", i);

                                        info!("  translation: {:?}", ph.translation());

                                        control_state.photo_ui_text = ph.to_string();

                                        control_state.active_drag =
                                        Some(Drag {
                                            mouse_start: *position,
                                            photo_start: ph.translation(),
                                            photo_index: i,
                                        });
                                        break;
                                    }
                                }
                            },
                            State::Released => control_state.active_drag = None,
                        },

                    MouseButton::Left =>
                        match control_state.active_mouse_tool {
                            MouseTool::SelectPhoto => {

                                control_state.selected_photo_index = None;

                                for (i, ph) in photos.iter().enumerate() {
                                    if ph.contains(world_coords) {
                                        info!("clicked on photos[{}]", i);

                                        info!("  translation: {:?}", ph.translation());

                                        control_state.photo_ui_text = ph.to_string();

                                        control_state.selected_photo_index = Some(i);
                                        break;
                                    }
                                }

                            }
                            MouseTool::RotationPoint =>
                                match *state {
                                    State::Pressed => {
                                        control_state.active_rotation_point =
                                        Some(RotationPoint {
                                            point: world_coords,
                                        });
                                    },
                                    _ => {},
                                },
                            MouseTool::DragToRotate =>
                                match *state {
                                    State::Pressed => {
                                        control_state.active_rotate_drag =
                                            control_state.selected_photo_index.map(|index| {
                                                RotateDrag {
                                                    mouse_start: world_coords,
                                                    mouse_coords: world_coords,
                                                    translate_start: photos[index].translation(),
                                                    rotate_start: photos[index].rotation(),
                                                    photo_index: index,
                                                }
                                            });
                                    },
                                    State::Released => control_state.active_rotate_drag = None,
                                }

                        }

                }
            },
            Event::MouseMotion {position, handled, ..} => {

                //cursor location debug info
                {
                    let pixel_coords = PixelCoords {x: position.0, y: position.1};
                    let world_coords = viewport_geometry.pixels_to_world(&pixel_coords);

                    control_state.mouse_location_ui_text =
                    format!("pixel_coords: {:?}\nworld_coords: {:?}", pixel_coords, world_coords);
                }

                if *handled {break};

                if let Some(ref mut pan) = control_state.active_pan {
                //    info!("mouse delta: {:?} {:?}", delta.0, delta.1);
                //    info!("mouse position: {:?} {:?}", position.0, position.1);
                    redraw = true;

                    viewport_geometry.camera_position.x = pan.camera_start.x as f64 - ((position.0 - pan.mouse_start.0) * viewport_geometry.world_units_per_pixel());
                    viewport_geometry.camera_position.y = pan.camera_start.y as f64 + ((position.1 - pan.mouse_start.1) * viewport_geometry.world_units_per_pixel());
                }

                if let Some(ref mut drag) = control_state.active_drag {

                    redraw = true;

                    let new_translation = WorldCoords {
                        x: drag.photo_start.x as f64 + ((position.0 - drag.mouse_start.0) * viewport_geometry.world_units_per_pixel()),
                        y: drag.photo_start.y as f64 - ((position.1 - drag.mouse_start.1) * viewport_geometry.world_units_per_pixel()),
                    };

                    photos[drag.photo_index].set_translation(new_translation);
                }

                if let Some(ref mut rotate_drag) = control_state.active_rotate_drag {

                    if let Some(ref rp) = control_state.active_rotation_point {

                        redraw = true;

                        let world_coords =
                            viewport_geometry.pixels_to_world(&PixelCoords{x: position.0, y: position.1});

                        rotate_drag.mouse_coords = world_coords; //update current mouse coords

                        let start = Vec2::new(rotate_drag.mouse_start.x as f32, rotate_drag.mouse_start.y as f32);
                        let axis = Vec2::new(rp.point.x as f32, rp.point.y as f32);
                        let drag = Vec2::new(world_coords.x as f32, world_coords.y as f32);

                        let axis_to_start = start - axis;
                        let axis_to_drag = drag - axis;

                        let drag_angle: cgmath::Deg<f32> = axis_to_start.angle(axis_to_drag).into();
                        let drag_angle = drag_angle.0;

                        //reset to values from start of rotation before rotate_around_point
                        photos[rotate_drag.photo_index].set_rotation(rotate_drag.rotate_start);
                        photos[rotate_drag.photo_index].set_translation(rotate_drag.translate_start);
                        photos[rotate_drag.photo_index].rotate_around_point(drag_angle, rp.point);
                    }
                }
            },
            Event::MouseWheel {delta, position, handled, ..} => {
                info!("{:?}", delta);

                if *handled {break};

                redraw = true;

                let pixel_coords = PixelCoords{x: position.0, y: position.1};
                let screen_coords = viewport_geometry.convert_pixel_to_screen(&pixel_coords);

                info!("cursor_screen {:?},{:?}", screen_coords.x, screen_coords.y);

                //center the zoom action on the cursor
                let to_cursor = viewport_geometry.convert_screen_to_world_at_origin(&screen_coords);
                viewport_geometry.camera_position.x += to_cursor.x;
                viewport_geometry.camera_position.y += to_cursor.y;

                //un-reverse direction in web mode (not sure why it's backwards)
                match (delta.1 > 0.0, cfg!(target_arch = "wasm32")) {
                    (true, true) | (false, false) => viewport_geometry.zoom_out(),
                    (true, false) | (false, true) => viewport_geometry.zoom_in(),
                }

                //and translate back, at the new zoom level
                let to_cursor = viewport_geometry.convert_screen_to_world_at_origin(&screen_coords);
                viewport_geometry.camera_position.x -= to_cursor.x;
                viewport_geometry.camera_position.y -= to_cursor.y;

                camera.set_orthographic_projection(viewport_geometry.width_in_world_units() as f32,
                                                   viewport_geometry.height_in_world_units() as f32,
                                                   10.0).unwrap();
            },
            Event::Key { state, kind, handled, ..} => {
                if *handled {break};

                if *kind == Key::S && *state == State::Pressed
                {
                    redraw = true;
                    control_state.dewarp_shader = match control_state.dewarp_shader {
                        DewarpShader::NoMorph => DewarpShader::Dewarp1,
                        DewarpShader::Dewarp1 => DewarpShader::Dewarp2,
                        DewarpShader::Dewarp2 => DewarpShader::NoMorph,
                    };
                }
            },
            _ => {},
        }
    }

    redraw
}