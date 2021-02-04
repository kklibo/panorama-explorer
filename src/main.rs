
use three_d::*;
use log::info;

mod read_pto;



fn load_mesh_from_filepath(gl: &Gl, renderer: &PhongDeferredPipeline, loaded: &mut Loaded, jpg_filepath: &str) -> PhongDeferredMesh {

    let mut square_cpu_mesh = CPUMesh {
        positions: square_positions(),
        uvs: Some(square_uvs()),
        ..Default::default()
    };
    square_cpu_mesh.compute_normals();
    let square_material = PhongMaterial {
        color_source: ColorSource::Texture(std::rc::Rc::new(texture::Texture2D::new_with_u8(&gl, Interpolation::Linear, Interpolation::Linear,
                                                                                            None, Wrapping::ClampToEdge, Wrapping::ClampToEdge,
                                                                                            &Loader::get_image(loaded, jpg_filepath).unwrap()).unwrap())),
        ..Default::default()
    };

    renderer.new_mesh(&square_cpu_mesh, &square_material).unwrap()
}


fn main() {

    if cfg!(not(target_arch = "wasm32")) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    let mut window = Window::new_default("panorama_tool").unwrap();
    let (width, height) = window.framebuffer_size();
    let gl = window.gl();

    // Renderer
    let mut renderer = PhongDeferredPipeline::new(&gl).unwrap();
    let mut camera =
        Camera::new_perspective(&gl,
                                vec3(0.0, 0.0, 5.0),
                                vec3(0.0, 0.0, 0.0),
                                vec3(0.0, 1.0, 0.0),
                                degrees(30.0),
                                width as f32 / height as f32, 0.1, 1000.0);

    let jpg_filepath = "test_photos/DSC_9479_6_25.JPG";

    Loader::load(&[jpg_filepath], move |loaded|
    {
        let square_mesh = load_mesh_from_filepath(&gl, &renderer, loaded, jpg_filepath);

        let ambient_light = AmbientLight::new(&gl, 0.4, &vec3(1.0, 1.0, 1.0)).unwrap();
        let directional_light = DirectionalLight::new(&gl, 1.0, &vec3(1.0, 1.0, 1.0), &vec3(0.0, -1.0, -1.0)).unwrap();

        // main loop
        let mut rotating = false;
        window.render_loop(move |frame_input|
        {
            camera.set_size(frame_input.screen_width as f32, frame_input.screen_height as f32);

            for event in frame_input.events.iter() {
                match event {
                    Event::MouseClick {state, button, ..} => {
                        rotating = *button == MouseButton::Left && *state == State::Pressed;
                    },
                    Event::MouseMotion {delta} => {
                        if rotating {
                            camera.rotate(delta.0 as f32, delta.1 as f32);
                        }
                    },
                    Event::MouseWheel {delta} => {
                        camera.zoom(*delta as f32);
                    },
                    Event::Key { state, kind } => {
                        if kind == "R" && *state == State::Pressed
                        {
                            renderer.next_debug_type();
                            info!("{:?}", renderer.debug_type());
                        }
                    }
                }
            }

            // draw
            // Geometry pass
            renderer.geometry_pass(width, height, &|| {

                //temporary test hardcode
                let t1 = Mat4::from_nonuniform_scale(460f32,307f32,1f32);
                let t2 = Mat4::from_scale(1f32/460f32).concat(&t1);
                let transformation = t2;

                square_mesh.render_geometry(&transformation, &camera)?;
                Ok(())
            }).unwrap();

            Screen::write(&gl, 0, 0, width, height, Some(&vec4(0.2, 0.2, 0.2, 1.0)), Some(1.0), || {
                renderer.light_pass(&camera, Some(&ambient_light), &[&directional_light], &[], &[])?;
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
