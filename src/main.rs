use std::rc::Rc;

use three_d::*;
use log::info;

mod viewport_geometry;
mod read_pto;
mod photo;

use viewport_geometry::{ViewportGeometry, PixelCoords, WorldCoords};
use photo::{Photo, convert_photo_px_to_world};

pub struct LoadedImageMesh {

    pub mesh: Mesh,
    pub texture_2d: Texture2D,
}

fn load_mesh_from_filepath(context: &Context, loaded: &mut Loaded, image_filepath: &str) -> LoadedImageMesh {

    let mut cpu_mesh = CPUMesh {
        positions: square_positions(),
        uvs: Some(square_uvs()),

        ..Default::default()
    };
    cpu_mesh.compute_normals();

    let mut cpu_texture = loaded.image(image_filepath).unwrap();
    cpu_texture.min_filter = Interpolation::Nearest;
    cpu_texture.mag_filter = Interpolation::Nearest;
    cpu_texture.mip_map_filter = None;
    cpu_texture.wrap_s = Wrapping::ClampToEdge;
    cpu_texture.wrap_t = Wrapping::ClampToEdge;
    cpu_texture.wrap_r = Wrapping::ClampToEdge;

    let texture_2d = Texture2D::new_with_u8(&context, &cpu_texture).unwrap();

    let mesh = Mesh::new(&context, &cpu_mesh).unwrap();

    LoadedImageMesh {mesh, texture_2d}
}

fn color_mesh(context: &Context) -> Mesh {

    let mut cpu_mesh = CPUMesh {
        positions: hourglass_positions(),

        ..Default::default()
    };
    cpu_mesh.compute_normals();

    let mesh = Mesh::new(&context, &cpu_mesh).unwrap();

    mesh
}

fn main() {

    if cfg!(not(target_arch = "wasm32")) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    let window = Window::new("panorama_tool", None).unwrap();
    let context = window.gl();

    let mut viewport_geometry = ViewportGeometry::try_new(
        WorldCoords{x:0.0, y:0.0},
        1_f64,
        10_u32,
        1_u32,
        15_u32,
        window.viewport().width,
        window.viewport().height,
    ).unwrap();


    // Renderer
    let mut camera =
    CameraControl::new(
        Camera::new_orthographic(&context,
                                 vec3(0.0, 0.0, 5.0),
                                 vec3(0.0, 0.0, 0.0),
                                 vec3(0.0, 1.0, 0.0),
                                 viewport_geometry.width_in_world_units() as f32,
                                 viewport_geometry.height_in_world_units() as f32,
                                 10.0).unwrap()
    );

    let mut gui = three_d::GUI::new(&context).unwrap();


    //let pto_file = "test_photos/test.pto";
    let pto_file = "test_photos/DSC_9108_12_5 - DSC_9109_12_5.pto";

    let filepaths = [
        pto_file,
    //    "test_photos/test1_border.jpg",
    //    "test_photos/test2_border.jpg",
    //    "test_photos/test1.jpg",
    //    "test_photos/test2.jpg",
        "test_photos/DSC_9108_12_5.JPG",
        "test_photos/DSC_9109_12_5.JPG",
    ];

    Loader::load(&filepaths, move |loaded|
    {

        let file_u8 = loaded.bytes(pto_file).unwrap();
        let s = std::str::from_utf8(file_u8).unwrap();

        let pairs = read_pto::read_control_point_pairs(s).unwrap();

        for (ref cp1, ref cp2) in &(*pairs) {
            info!("({:?}, {:?})", cp1, cp2);
        }

        info!("pairs size: {}", (*pairs).len());

        let image0_control_points =
            pairs.iter().filter_map(|(cp1, _cp2)| {
                match cp1.image_id {
                    0 => Some(Vec3::new(cp1.x_coord as f32, cp1.y_coord as f32, 0 as f32)),
                    _ => None,
                }
            }).collect::<Vec<Vec3>>();

        for &Vec3{x,y,z} in &image0_control_points {
            info!("({:?}, {:?}, {:?})", x,y,z);
        }

        let image1_control_points =
            pairs.iter().filter_map(|(_cp1, cp2)| {
                match cp2.image_id {
                    1 => Some(Vec3::new(cp2.x_coord as f32, cp2.y_coord as f32, 0 as f32)),
                    _ => None,
                }
            }).collect::<Vec<Vec3>>();

        let meshes: Vec<Rc<LoadedImageMesh>> = filepaths.iter().skip(1).map(|x| {
            Rc::new(load_mesh_from_filepath(&context, loaded, x))
        }).collect();

        let mut photos = [
            Photo::from_loaded_image_mesh(meshes[0].clone()),
            Photo::from_loaded_image_mesh(meshes[1].clone()),
        ];
        photos[1].set_translation(WorldCoords{x: 500.0, y: 0.0});

        let color_mesh = color_mesh(&context);

        let         texture_program = MeshProgram::new(&context, include_str!(        "texture.frag")).unwrap();
        let  texture_dewarp_program = MeshProgram::new(&context, include_str!( "texture_dewarp.frag")).unwrap();
        let texture_dewarp2_program = MeshProgram::new(&context, include_str!("texture_dewarp2.frag")).unwrap();
        let           color_program = MeshProgram::new(&context, include_str!(          "color.frag")).unwrap();


        // main loop

        #[derive(PartialEq, Debug)]
        enum DewarpShader {
            NoMorph,
            Dewarp1,
            Dewarp2,
        }
        let mut dewarp_shader = DewarpShader::NoMorph;

        struct Pan {
            mouse_start: (f64,f64),
            camera_start: Vec3,
        }
        let mut active_pan: Option<Pan> = None;

        struct Drag {
            mouse_start: (f64,f64),
            photo_start: WorldCoords,
            photo_index: usize, //replace this
        }
        let mut active_drag: Option<Drag> = None;

        struct RotationPoint {
            point: WorldCoords,
            translate_start: WorldCoords,
            rotate_start: f32,
        }
        let mut active_rotation_point: Option<RotationPoint> = None;

        struct RotateDrag {
            mouse_start: WorldCoords,
            rotate_start: f32, //degrees
            //photo_index: usize, //replace this
        }
        let mut active_rotate_drag: Option<RotateDrag> = None;

        #[derive(Debug, PartialEq)]
        enum MouseTool {
            RotationPoint,
            DragToRotate,
        }
        let mut active_mouse_tool: MouseTool = MouseTool::RotationPoint;

        let mut dewarp_strength: f32 = 0.0;
        let mut debug_rotation: f32 = 0.0;

        let mut mouse_click_ui_text= "".to_string();
        let mut photo_ui_text= "".to_string();

        window.render_loop(move |mut frame_input|
        {
            let update_shader_uniforms = |dewarp_strength: &f32| {
                texture_dewarp_program.use_uniform_float("strength", dewarp_strength).unwrap();
            };

            viewport_geometry.set_pixel_dimensions(frame_input.viewport.width, frame_input.viewport.height).unwrap();

            let mut redraw = frame_input.first_frame;
            redraw |= camera.set_aspect(frame_input.viewport.aspect()).unwrap();
            camera.set_orthographic_projection(viewport_geometry.width_in_world_units() as f32,
                                               viewport_geometry.height_in_world_units() as f32,
                                               10.0).unwrap();


            let mut panel_width = frame_input.viewport.width / 10;
            redraw |= gui.update(&mut frame_input, |gui_context| {

                use three_d::egui::*;
                SidePanel::left("side_panel", panel_width as f32).show(gui_context, |ui| {
                    ui.heading("panorama_tool");
                    ui.separator();

                    ui.heading("Left-click Tool:");
                    ui.radio_value(&mut active_mouse_tool, MouseTool::RotationPoint, format!("{:?}", MouseTool::RotationPoint));
                    ui.radio_value(&mut active_mouse_tool, MouseTool::DragToRotate,  format!("{:?}", MouseTool::DragToRotate ));
                    ui.separator();

                    ui.heading("Lens Correction");

                    let slider = Slider::f32(&mut dewarp_strength, 0.0..=10.0)
                        .text("dewarp strength")
                        .clamp_to_range(true);

                    if ui.add(slider).changed() {
                        update_shader_uniforms(&dewarp_strength);
                    }
                    ui.separator();

                    ui.heading("rotation test");
                    let slider = Slider::f32(&mut debug_rotation, -1.0..=1.0)
                        .text("angle")
                        .clamp_to_range(true);
                    if ui.add(slider).changed() {

                        if let Some(ref rp) = active_rotation_point {
                            //reset to values from start of rotation before rotate_around_point
                            photos[1].set_rotation(rp.rotate_start);
                            photos[1].set_translation(rp.translate_start);
                            photos[1].rotate_around_point(debug_rotation, rp.point);
                        }
                    }
                    ui.separator();

                    ui.heading("Dewarp Shader");
                    ui.radio_value(&mut dewarp_shader, DewarpShader::NoMorph, format!("{:?}", DewarpShader::NoMorph));
                    ui.radio_value(&mut dewarp_shader, DewarpShader::Dewarp1, format!("{:?}", DewarpShader::Dewarp1));
                    ui.radio_value(&mut dewarp_shader, DewarpShader::Dewarp2, format!("{:?}", DewarpShader::Dewarp2));
                    ui.separator();

                    ui.heading("Mouse Info");
                    ui.label(&mouse_click_ui_text);
                    ui.separator();

                    ui.heading("Photo Info");
                    ui.label(&photo_ui_text);
                    ui.separator();


                });
                panel_width = (gui_context.used_size().x * gui_context.pixels_per_point()) as usize;
            }).unwrap();


            for event in frame_input.events.iter() {
                match event {
                    Event::MouseClick {state, button, position, handled, ..} => {
                        info!("MouseClick: {:?}", event);
                        mouse_click_ui_text = format!("MouseClick: {:#?}", event);

                        if *handled {break};

                        let world_coords =
                        viewport_geometry.pixels_to_world(&PixelCoords{x: position.0, y: position.1});
                        info!("  WorldCoords: {:?}", world_coords);


                        match *button {

                            MouseButton::Middle => active_pan =
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

                                        active_drag = None;

                                        for (i, ph) in photos.iter().enumerate() {
                                            if ph.contains(world_coords) {
                                                info!("clicked on photos[{}]", i);

                                                info!("  translation: {:?}", ph.translation());

                                                photo_ui_text = ph.to_string();

                                                active_drag =
                                                Some(Drag {
                                                    mouse_start: *position,
                                                    photo_start: ph.translation(),
                                                    photo_index: i,
                                                });
                                                break;
                                            }
                                        }
                                    },
                                    State::Released => active_drag = None,
                                },

                            MouseButton::Left =>
                                match active_mouse_tool {
                                    MouseTool::RotationPoint =>
                                        match *state {
                                            State::Pressed => {
                                                active_rotation_point =
                                                Some(RotationPoint {
                                                    point: world_coords,
                                                    translate_start: photos[1].translation(),
                                                    rotate_start: photos[1].rotation(),
                                                });
                                                debug_rotation = 0.0;
                                            },
                                            _ => {},
                                        },
                                    MouseTool::DragToRotate =>
                                        match *state {
                                            State::Pressed => {
                                                active_rotate_drag =
                                                Some(RotateDrag {
                                                    mouse_start: world_coords,
                                                    rotate_start: photos[1].rotation(),
                                                });
                                            },
                                            State::Released => active_rotate_drag = None,
                                        }

                                }


                        }
                    },
                    Event::MouseMotion {position, handled, ..} => {
                        if *handled {break};

                        if let Some(ref mut pan) = active_pan {
                        //    info!("mouse delta: {:?} {:?}", delta.0, delta.1);
                        //    info!("mouse position: {:?} {:?}", position.0, position.1);
                            redraw = true;

                            viewport_geometry.camera_position.x = pan.camera_start.x as f64 - ((position.0 - pan.mouse_start.0) * viewport_geometry.world_units_per_pixel());
                            viewport_geometry.camera_position.y = pan.camera_start.y as f64 + ((position.1 - pan.mouse_start.1) * viewport_geometry.world_units_per_pixel());
                        }

                        if let Some(ref mut drag) = active_drag {

                            redraw = true;

                            let new_translation = WorldCoords {
                                x: drag.photo_start.x as f64 + ((position.0 - drag.mouse_start.0) * viewport_geometry.world_units_per_pixel()),
                                y: drag.photo_start.y as f64 - ((position.1 - drag.mouse_start.1) * viewport_geometry.world_units_per_pixel()),
                            };

                            photos[drag.photo_index].set_translation(new_translation);
                        }

                        if let Some(ref rotate_drag) = active_rotate_drag {

                            if let Some(ref rp) = active_rotation_point {

                                redraw = true;

                                let world_coords =
                                    viewport_geometry.pixels_to_world(&PixelCoords{x: position.0, y: position.1});

                                let start = Vec2::new(rotate_drag.mouse_start.x as f32, rotate_drag.mouse_start.y as f32);
                                let axis = Vec2::new(rp.point.x as f32, rp.point.y as f32);
                                let drag = Vec2::new(world_coords.x as f32, world_coords.y as f32);

                                let axis_to_start = start - axis;
                                let axis_to_drag = drag - axis;

                                let drag_angle: cgmath::Deg<f32> = axis_to_start.angle(axis_to_drag).into();
                                let drag_angle = drag_angle.0;


                                //reset to values from start of rotation before rotate_around_point
                                photos[1].set_rotation(rp.rotate_start);
                                photos[1].set_translation(rp.translate_start);
                                photos[1].rotate_around_point(drag_angle, rp.point);

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
                            dewarp_shader = match dewarp_shader {
                                DewarpShader::NoMorph => DewarpShader::Dewarp1,
                                DewarpShader::Dewarp1 => DewarpShader::Dewarp2,
                                DewarpShader::Dewarp2 => DewarpShader::NoMorph,
                            };
                        }

                        if *kind == Key::PageUp && *state == State::Pressed
                        {
                            redraw = true;
                            dewarp_strength += 0.1;
                            update_shader_uniforms(&dewarp_strength);
                        }

                        if *kind == Key::PageDown && *state == State::Pressed
                        {
                            redraw = true;
                            dewarp_strength -= 0.1;
                            update_shader_uniforms(&dewarp_strength);
                        }
                    },
                    _ => {},
                }
            }

            camera.set_view(
                vec3(viewport_geometry.camera_position.x as f32, viewport_geometry.camera_position.y as f32, 5.0),
                vec3(viewport_geometry.camera_position.x as f32, viewport_geometry.camera_position.y as f32, 0.0),
                vec3(0.0, 1.0, 0.0)
            ).unwrap();


            //temp: window resize needs to trigger redraw, anything else?
            redraw |= true;
            //

            // draw
            if redraw {
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

                        let program = match dewarp_shader
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

                    if let Some(ref rp) = active_rotation_point {
                        let t1 = Mat4::from_nonuniform_scale(10.0, 10.0, 1.0);
                        let t1 = Mat4::from_angle_z(cgmath::Deg(-45.0)).concat(&t1);
                        let t1 = Mat4::from_translation(Vec3::new(0.0, 0.0, 1.0)).concat(&t1);

                        let t1 = Mat4::from_translation(Vec3::new(rp.point.x as f32, rp.point.y as f32, 0.0)).concat(&t1);

                        color_program.use_uniform_vec4("color", &Vec4::new(0.8, 0.8, 0.2, 0.5)).unwrap();
                        color_mesh.render(&color_program, render_states, frame_input.viewport, &t1, &camera)?;
                    }


                    gui.render().unwrap();

                    Ok(())
                }).unwrap();


                //set entire display buffer alpha to 1.0: prevents web browser pass-through transparency problem
                let clear_alpha = ClearState {
                    red: None,
                    green: None,
                    blue: None,
                    alpha: Some(1.0),
                    depth: None,
                };
                Screen::write(&context, &clear_alpha, || { Ok(()) }).unwrap();
            }

            FrameOutput {swap_buffers: redraw, ..Default::default()}

        }).unwrap();
    });
}

fn square_positions() -> Vec<f32> {
    vec![
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
        0.5, 0.5, 0.0,
        0.5, 0.5, 0.0,
        -0.5, 0.5, 0.0,
        -0.5, -0.5, 0.0,
    ]
}

fn square_uvs() -> Vec<f32> {
    vec![
        0.0, 0.0,
        1.0, 0.0,
        1.0, 1.0,
        1.0, 1.0,
        0.0, 1.0,
        0.0, 0.0,
    ]
}

fn hourglass_positions() -> Vec<f32> {
    vec![
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
        0.0, 0.0, 0.0,
        0.5, 0.5, 0.0,
        -0.5, 0.5, 0.0,
        0.0, 0.0, 0.0,
    ]
}
