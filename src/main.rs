
use three_d::*;
use log::info;

mod zoom;
mod read_pto;

struct LoadedImageMesh {

    pub mesh: PhongDeferredMesh,
    pub pixel_width: u32,
    pub pixel_height: u32,
}

fn load_mesh_from_filepath(context: &Context, loaded: &mut Loaded, image_filepath: &str) -> LoadedImageMesh {

    let mut cpu_mesh = CPUMesh {
        positions: square_positions(),
        uvs: Some(square_uvs()),

        ..Default::default()
    };
    cpu_mesh.compute_normals();

    let image = Loader::get_image(loaded, image_filepath).unwrap();

    let material = PhongMaterial {
        color_source: ColorSource::Texture(std::rc::Rc::new(
            texture::Texture2D::new_with_u8(&context,
                Interpolation::Linear, Interpolation::Linear,
                None, Wrapping::ClampToEdge, Wrapping::ClampToEdge,
                &image).unwrap())),

        ..Default::default()
    };

    let mesh = PhongDeferredMesh::new(&context, &cpu_mesh, &material).unwrap();

    LoadedImageMesh {mesh, pixel_width: image.width, pixel_height: image.height}
}

fn main() {

    if cfg!(not(target_arch = "wasm32")) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    let mut window = Window::new("panorama_tool", None).unwrap();
    let context = window.gl();

    let mut zoom = zoom::Zoom {
        scale: 1_f64,
        value: 10_u32,
        min: 1_u32,
        max: 15_u32,
    };


    // Renderer
    let mut pipeline = PhongDeferredPipeline::new(&context).unwrap();
    let mut camera =
        Camera::new_orthographic(&context,
                                vec3(0.0, 0.0, 5.0),
                                vec3(0.0, 0.0, 0.0),
                                vec3(0.0, 1.0, 0.0),
                                zoom.gl_units_width(),
                                 zoom.gl_units_height(window.viewport().aspect()),
                                 10.0);


    let jpg_filepaths = [
        "test_photos/DSC_9108_12_5.JPG",
        "test_photos/DSC_9109_12_5.JPG"
    ];

    Loader::load(&jpg_filepaths, move |loaded|
    {
        let meshes = jpg_filepaths.iter().map(|x| {
            load_mesh_from_filepath(&context, loaded, x)
        }).collect::<Vec<_>>();

        let ambient_light = AmbientLight {intensity: 0.4, color: vec3(1.0, 1.0, 1.0)};
        let directional_light = DirectionalLight::new(&context, 1.0, &vec3(1.0, 1.0, 1.0), &vec3(0.0, -1.0, -1.0)).unwrap();

        // main loop

        struct Pan {
            mouse_start: (f64,f64),
            camera_start: Vec3,
        }
        let mut active_pan: Option<Pan> = None;

        let mut mouse_position = (0_f64,0_f64); //temp

        window.render_loop(move |frame_input|
        {
            camera.set_aspect(frame_input.viewport.aspect());
            camera.set_orthographic_projection(zoom.gl_units_width(),
                                               zoom.gl_units_height(frame_input.viewport.aspect()),
                                               10.0);

            for event in frame_input.events.iter() {
                match event {
                    Event::MouseClick {state, button, position} => {
                        info!("MouseClick: mouse position: {:?} {:?}", position.0, position.1);
                        mouse_position = *position;

                        active_pan =
                        match *button == MouseButton::Left && *state == State::Pressed {
                            true => Some(Pan {
                                mouse_start: *position,
                                camera_start: camera.position().clone(),
                            }),
                            false => None,
                        };

                    },
                    Event::MouseMotion {delta, position} => {

                        mouse_position = *position;

                        if let Some(ref mut pan) = active_pan {
                            info!("mouse delta: {:?} {:?}", delta.0, delta.1);
                            info!("mouse position: {:?} {:?}", position.0, position.1);

                            let camera_position_x = pan.camera_start.x - ((position.0 - pan.mouse_start.0) * zoom.gl_units_per_pixel(frame_input.viewport.width)) as f32;
                            let camera_position_y = pan.camera_start.y + ((position.1 - pan.mouse_start.1) * zoom.gl_units_per_pixel(frame_input.viewport.width)) as f32;

                            camera.set_view(
                                vec3(camera_position_x as f32, camera_position_y as f32, 5.0),
                                vec3(camera_position_x as f32, camera_position_y as f32, 0.0),
                                vec3(0.0, 1.0, 0.0)
                            );
                        }
                    },
                    Event::MouseWheel {delta} => {
                        info!("{:?}", delta);

                        if frame_input.viewport.width  <= 0 {panic!("non-positive viewport width" );}
                        if frame_input.viewport.height <= 0 {panic!("non-positive viewport height");}
                        let cursor_screen_x = mouse_position.0 / frame_input.viewport.width as f64;
                        let cursor_screen_y = 1_f64-(mouse_position.1 / frame_input.viewport.height as f64);

                        info!("cursor_screen {:?},{:?}", cursor_screen_x, cursor_screen_y);

                        match (*delta > 0.0, cfg!(target_arch = "wasm32")) {
                            (true, true) | (false, false) => {
                                camera.translate(&Vec3::new(
                                    zoom.gl_units_width()*(cursor_screen_x-0.5) as f32,
                                    zoom.gl_units_height(frame_input.viewport.aspect())*(cursor_screen_y-0.5) as f32,
                                    0.0));
                                zoom.zoom_out();
                                camera.translate(&Vec3::new(
                                    -zoom.gl_units_width()*(cursor_screen_x-0.5) as f32,
                                    -zoom.gl_units_height(frame_input.viewport.aspect())*(cursor_screen_y-0.5) as f32,
                                    0.0));
                            },
                            (true, false) | (false, true) => {
                                camera.translate(&Vec3::new(
                                    zoom.gl_units_width()*(cursor_screen_x-0.5) as f32,
                                    zoom.gl_units_height(frame_input.viewport.aspect())*(cursor_screen_y-0.5) as f32,
                                    0.0));
                                zoom.zoom_in();
                                camera.translate(&Vec3::new(
                                    -zoom.gl_units_width()*(cursor_screen_x-0.5) as f32,
                                    -zoom.gl_units_height(frame_input.viewport.aspect())*(cursor_screen_y-0.5) as f32,
                                    0.0));
                            },
                        }

                        camera.set_orthographic_projection(zoom.gl_units_width(),
                                                           zoom.gl_units_height(frame_input.viewport.aspect()),
                                                           10.0);
                    },
                    Event::Key { state, kind } => {
                        if kind == "R" && *state == State::Pressed
                        {
                            pipeline.next_debug_type();
                            info!("{:?}", pipeline.debug_type());
                        }
                    }
                }
            }

            // draw
            // Geometry pass
            pipeline.geometry_pass(frame_input.viewport.width, frame_input.viewport.height, &|| {

                let t1 = Mat4::from_nonuniform_scale(meshes[0].pixel_width as f32,meshes[0].pixel_height as f32,1f32);
                //let t2 = Mat4::from_scale(1f32/meshes[0].pixel_width as f32).concat(&t1);
                //let transformation = t2;

                meshes[0].mesh.render_geometry(RenderStates {cull: CullType::Back, ..Default::default()},
                                               frame_input.viewport, &t1, &camera)?;

                let t1 = Mat4::from_nonuniform_scale(meshes[1].pixel_width as f32,meshes[1].pixel_height as f32,1f32);
                let t1= Mat4::from_translation(cgmath::Vector3::new(1000f32, 0f32, 0f32)).concat(&t1);
                //let t2 = Mat4::from_scale(1f32/meshes[1].pixel_width as f32).concat(&t1);
                //let transformation = t2;

                meshes[1].mesh.render_geometry(RenderStates {cull: CullType::Back, ..Default::default()},
                                               frame_input.viewport, &t1, &camera)?;

                Ok(())
            }).unwrap();

            Screen::write(&context, Some(&vec4(0.2, 0.2, 0.2, 1.0)), Some(1.0), || {
                pipeline.light_pass(frame_input.viewport, &camera, Some(&ambient_light), &[&directional_light], &[], &[])?;
                Ok(())
            }).unwrap();

        }).unwrap();
    });
}

fn square_positions() -> Vec<f32> {
    vec![
        -1.0, -1.0, 0.0,
        1.0, -1.0, 0.0,
        1.0, 1.0, 0.0,
        1.0, 1.0, 0.0,
        -1.0, 1.0, 0.0,
        -1.0, -1.0, 0.0,
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
