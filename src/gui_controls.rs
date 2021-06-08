use three_d::camera::CameraControl;
use three_d::frame::FrameInput;
use three_d::frame::{Event, MouseButton, State, Key};
use three_d::gui::GUI;
use three_d::egui::{Window, Button, CollapsingHeader};
use three_d::math::{Vec2, InnerSpace};

use log::info;

use crate::viewport_geometry::{ViewportGeometry, PixelCoords, WorldCoords};
use crate::control_state::{ControlState, MouseTool, DewarpShader, Pan, Drag, RotateDrag, RotationPoint, UiMode};
use crate::photo::Photo;
use crate::entities::Entities;

pub fn run_gui_controls(
    frame_input: &mut FrameInput,
    gui: &mut GUI,
    control_state: &mut ControlState,
    viewport_geometry: &mut ViewportGeometry,
    entities: &mut Entities,
) -> bool {

    let redraw = gui.update(frame_input, |gui_context| {

        let window = Window::new("panorama tool").scroll(true);

        window.show(gui_context, |ui| {

            ui.horizontal(|ui| {
                ui.selectable_value(&mut control_state.ui_mode, UiMode::Browse, "Browse");
                ui.selectable_value(&mut control_state.ui_mode, UiMode::Edit, "Edit");
            });

            match control_state.ui_mode {

                UiMode::Browse => {

                    ui.label("browse mode");
                },
                UiMode::Edit => {

                    ui.heading("Left-click Tool:");
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::PanView, format!("{:?}", MouseTool::PanView));
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::DragPhoto, format!("{:?}", MouseTool::DragPhoto));
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::SelectPhoto, format!("{:?}", MouseTool::SelectPhoto));
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::RotationPoint, format!("{:?}", MouseTool::RotationPoint));
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::DragToRotate, format!("{:?}", MouseTool::DragToRotate));
                    ui.radio_value(&mut control_state.active_mouse_tool, MouseTool::DragToRotateAllPhotos, format!("{:?}", MouseTool::DragToRotateAllPhotos));
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.add(Button::new("Zoom In")).clicked() {
                            viewport_geometry.zoom_in();
                        }
                        if ui.add(Button::new("Zoom Out")).clicked() {
                            viewport_geometry.zoom_out();
                        }
                    });
                    ui.separator();

                    ui.heading("Dewarp Shader");
                    ui.radio_value(&mut control_state.dewarp_shader, DewarpShader::NoMorph, format!("Off"));
                    ui.radio_value(&mut control_state.dewarp_shader, DewarpShader::Dewarp2, format!("On"));
                    ui.separator();

                    let mut photo_ui_text = "None".to_string();

                    if let Some(i) = control_state.selected_photo_index {
                        if let Some(ph) = entities.photos.get(i) {
                            photo_ui_text = format!(
                                "Photo {}\n\
                                Center:\n\
                                 x: {:.2}\n\
                                 y: {:.2}\n\
                                Rotation: {:.2}°",
                                i,
                                ph.orientation().translation().x,
                                ph.orientation().translation().y,
                                ph.orientation().rotation()
                            );
                        }
                    }
                    ui.heading("Selected Photo Info");
                    ui.label(&photo_ui_text);
                    ui.separator();

                    ui.label("demo");

                    if ui.add(Button::new("reset photos")).clicked() {

                        entities.set_photos_from_json_serde_string(&entities.reset_photos_string.clone()).unwrap();
                    }
                    if ui.add(Button::new("align photos")).clicked() {

                        entities.set_photos_from_json_serde_string(&entities.align_photos_string.clone()).unwrap();
                    }

                    ui.separator();

                    CollapsingHeader::new("Help")
                        .default_open(false)
                        .show(ui, |ui| {

                            ui.label(format!(
                                "Left Mouse: use tool\n\
                                Middle Mouse: pan view\n\
                                Scroll Wheel: zoom in/out\n\
                                Right Mouse: drag photo"
                                )
                            );

                        });


                    CollapsingHeader::new("Debug")
                        .default_open(false)
                        .show(ui, |ui| {

                        ui.heading("Mouse Location");
                        ui.label(&control_state.mouse_location_ui_text);
                        ui.separator();

                        ui.checkbox(&mut control_state.alignment_mode, "Alignment Mode (rendering)");
                        ui.separator();

                        ui.checkbox(&mut control_state.control_points_visible, "Show Control Points");
                        ui.separator();

                        ui.checkbox(&mut control_state.photo_borders_visible, "Show Photo Borders");
                        ui.separator();

                        if ui.add(Button::new("dump debug info")).clicked() {
                            for ph in &entities.photos {
                                info!("{}", serde_json::to_string(ph).unwrap());
                            }
                        }
                    });
                },
            }
        });
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

                //allow button releases in the UI to end drag actions
                // otherwise don't re-handle UI mouse clicks
                if *state != State::Released && *handled {break};

                let world_coords =
                viewport_geometry.pixels_to_world(&PixelCoords{x: position.0, y: position.1});

                //pan view click handler
                let pan_view = |control_state: &mut ControlState| {
                    control_state.active_pan =
                        match *state {
                            State::Pressed => {
                                Some(Pan {
                                    mouse_start: *position,
                                    camera_start: camera.position().clone(),
                                })
                            },
                            State::Released => None,
                        };
                };

                //drag photo click handler
                let drag_photo = |control_state: &mut ControlState| {
                    match *state {
                        State::Pressed => {

                            control_state.active_drag = None;

                            //only modify the selected photo (if there is one)
                            if let Some(i) = control_state.selected_photo_index {
                                if photos[i].orientation().contains(world_coords) {
                                        control_state.active_drag =
                                            Some(Drag {
                                                mouse_start: *position,
                                                photo_start: photos[i].orientation().translation(),
                                                photo_index: i,
                                            });
                                    }

                            }
                            //if no photo is selected, allow drags for any photo
                            else {
                                for (i, ph) in photos.iter().enumerate() {
                                    if ph.orientation().contains(world_coords) {
                                        control_state.active_drag =
                                            Some(Drag {
                                                mouse_start: *position,
                                                photo_start: ph.orientation().translation(),
                                                photo_index: i,
                                            });
                                        break;
                                    }
                                }
                            }
                        },
                        State::Released => control_state.active_drag = None,
                    };
                };


                match *button {

                    MouseButton::Middle => pan_view(control_state),

                    MouseButton::Right => drag_photo(control_state),

                    MouseButton::Left =>
                        match control_state.active_mouse_tool {
                            MouseTool::PanView => pan_view(control_state),

                            MouseTool::DragPhoto => drag_photo(control_state),

                            MouseTool::SelectPhoto => {

                                if *state == State::Pressed {

                                    //collect all photos which are under the cursor
                                    let clicked_photos: Vec<(usize, &Photo)> =
                                        photos.iter().enumerate().filter(|(_, ph)| {
                                            ph.orientation().contains(world_coords)
                                        }).collect();

                                    let next_photo =
                                        //if a photo is selected
                                        if let Some(selected) = control_state.selected_photo_index {

                                            //if one of these is the currently selected one
                                            // advance to the next one (by index):

                                            //skip until the selected photo is reached, or the end
                                            clicked_photos.iter().skip_while(|(i, _)| {
                                                selected != *i
                                            })

                                            //skip the selected photo, get the next one
                                            .skip(1).next()

                                        } else { None };

                                    //if next_photo is None:
                                    // a selected photo was not clicked on, or
                                    // the selected photo was the highest index that was clicked on
                                    //in either case, select the first photo that was clicked on (if any)
                                    control_state.selected_photo_index =
                                        next_photo.or(clicked_photos.first())
                                            .map(|(i, _)| *i);

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
                                                    translate_start: photos[index].orientation().translation(),
                                                    rotate_start: photos[index].orientation().rotation(),
                                                    photo_index: index,
                                                }
                                            });
                                    },
                                    State::Released => control_state.active_rotate_drag = None,
                                }
                            MouseTool::DragToRotateAllPhotos =>
                                match *state {
                                    State::Pressed => {
                                        control_state.active_rotate_all_photos_drag =

                                            //create a new active RotateDrag instance for every photo
                                            photos.iter().enumerate().map(|(index, p)| {
                                                RotateDrag {
                                                    mouse_start: world_coords,
                                                    mouse_coords: world_coords,
                                                    translate_start: p.orientation().translation(),
                                                    rotate_start: p.orientation().rotation(),
                                                    photo_index: index,
                                                }
                                            }).collect();
                                    },
                                    State::Released => control_state.active_rotate_all_photos_drag = Vec::new(),
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

                //photo rotation function, for DragToRotate + DragToRotateAllPhotos
                let mut rotate_photo = |rotate_drag: &mut RotateDrag, rp: &RotationPoint| {

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
                };

                if let Some(ref mut rotate_drag) = control_state.active_rotate_drag {

                    if let Some(ref rp) = control_state.active_rotation_point {

                        redraw = true;

                        rotate_photo(rotate_drag, rp);
                    }
                }

                if !control_state.active_rotate_all_photos_drag.is_empty() {

                    if let Some(ref rp) = control_state.active_rotation_point {

                        redraw = true;

                        for rotate_drag in &mut control_state.active_rotate_all_photos_drag {
                            rotate_photo(rotate_drag, rp);
                        }
                    }
                }
            },
            Event::MouseWheel {delta, position, handled, ..} => {

                if *handled {break};

                redraw = true;

                let pixel_coords = PixelCoords{x: position.0, y: position.1};
                let screen_coords = viewport_geometry.convert_pixel_to_screen(&pixel_coords);

                //center the zoom action on the cursor
                let to_cursor = viewport_geometry.convert_screen_to_world_at_origin(&screen_coords);
                viewport_geometry.camera_position.x += to_cursor.x;
                viewport_geometry.camera_position.y += to_cursor.y;

                match delta.1 > 0.0 {
                    true => viewport_geometry.zoom_in(),
                    false => viewport_geometry.zoom_out(),
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