
use three_d::*;
use log::info;

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


struct Zoom {
    pub scale: f64,
    pub value: u32,
    pub min: u32,
    pub max: u32,
}

impl Zoom {
    fn zoom_in(&mut self) {
        if self.value > self.min {
            self.value -= 1;
        }
    }
    fn zoom_out(&mut self) {
        if self.value < self.max {
            self.value += 1;
        }
    }
    fn get_size(&self) -> f64 {
        2_u32.pow(self.value) as f64 * self.scale
    }
}


fn main() {

    if cfg!(not(target_arch = "wasm32")) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    let mut window = Window::new("panorama_tool", None).unwrap();
    let (width, height) =
        if cfg!(target_arch = "wasm32") {
            (1280, 720) //temp hardcode for web canvas
        } else {
            (window.viewport().width, window.viewport().height)
        };

    let context = window.gl();

    let mut zoom = Zoom {
        scale: 1_f64,
        value: 10_u32,
        min: 1_u32,
        max: 100_u32,
    };

    fn get_gl_units_per_pixel(zoom: &Zoom, width: usize) -> f64 {
        if width == 0 {panic!("gl width = 0");}
        zoom.get_size() / width as f64
    };

    // Renderer
    let mut pipeline = PhongDeferredPipeline::new(&context).unwrap();
    let mut camera =
        Camera::new_orthographic(&context,
                                vec3(0.0, 0.0, 5.0),
                                vec3(0.0, 0.0, 0.0),
                                vec3(0.0, 1.0, 0.0),
                                zoom.get_size() as f32, zoom.get_size() as f32, 10.0);


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
        let mut panning = false;
        let mut pan_mouse_start = (0_f64, 0_f64); //temp
        let mut pan_camera_start = camera.position().clone(); //temp

        window.render_loop(move |frame_input|
        {
            for event in frame_input.events.iter() {
                match event {
                    Event::MouseClick {state, button, position} => {
                        info!("MouseClick: mouse position: {:?} {:?}", position.0, position.1);

                        panning = *button == MouseButton::Left && *state == State::Pressed;

                        if panning {
                            pan_mouse_start = *position;
                            pan_camera_start = camera.position().clone();
                        }
                    },
                    Event::MouseMotion {delta, position} => {
                        if panning {
                            info!("mouse delta: {:?} {:?}", delta.0, delta.1);
                            info!("mouse position: {:?} {:?}", position.0, position.1);

                            let camera_position_x = pan_camera_start.x - ((position.0 - pan_mouse_start.0) * get_gl_units_per_pixel(&zoom, width)) as f32;
                            let camera_position_y = pan_camera_start.y + ((position.1 - pan_mouse_start.1) * get_gl_units_per_pixel(&zoom, width)) as f32;

                            camera.set_view(
                                vec3(camera_position_x as f32, camera_position_y as f32, 5.0),
                                vec3(camera_position_x as f32, camera_position_y as f32, 0.0),
                                vec3(0.0, 1.0, 0.0)
                            );
                        }
                    },
                    Event::MouseWheel {delta} => {
                        info!("{:?}", delta);

                        match (*delta > 0.0, cfg!(target_arch = "wasm32")) {
                            (true, true) => zoom.zoom_out(),
                            (true, false) => zoom.zoom_in(),
                            (false, true) => zoom.zoom_in(),
                            (false, false) => zoom.zoom_out(),
                        }

                        camera.set_orthographic_projection(zoom.get_size() as f32, zoom.get_size() as f32, 10.0);
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
            pipeline.geometry_pass(width, height, &|| {

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
