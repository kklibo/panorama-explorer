
use three_d::*;

mod read_pto;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let screenshot_path = if args.len() > 1 { Some(args[1].clone()) } else {None};

    let mut window = Window::new_default("Texture").unwrap();
    let (width, height) = window.framebuffer_size();
    let gl = window.gl();

    // Renderer
    let mut renderer = PhongDeferredPipeline::new(&gl).unwrap();
    let mut camera = Camera::new_perspective(&gl, vec3(4.0, 1.5, 4.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 0.0),
                                             degrees(45.0), width as f32 / height as f32, 0.1, 1000.0);

    Loader::load(&["examples/assets/PenguinBaseMesh.obj", "examples/assets/PenguinBaseMesh.mtl",
        "examples/assets/penguin.png", "examples/assets/test_texture.jpg",
        "examples/assets/skybox_evening/back.jpg", "examples/assets/skybox_evening/front.jpg",
        "examples/assets/skybox_evening/top.jpg", "examples/assets/skybox_evening/left.jpg",
        "examples/assets/skybox_evening/right.jpg"], move |loaded|
    {
        let mut box_cpu_mesh = CPUMesh {
            positions: cube_positions(),
            ..Default::default()
        };
        box_cpu_mesh.compute_normals();
        let box_material = PhongMaterial {
            color_source: ColorSource::Texture(std::rc::Rc::new(texture::Texture2D::new_with_u8(&gl, Interpolation::Linear, Interpolation::Linear,
                                                                                                Some(Interpolation::Linear), Wrapping::Repeat, Wrapping::Repeat,
                                                                                                &Loader::get_image(loaded, "examples/assets/test_texture.jpg").unwrap()).unwrap())),
            ..Default::default()
        };
        let box_mesh = renderer.new_mesh(&box_cpu_mesh, &box_material).unwrap();

        let skybox = Skybox::new(&gl, &Loader::get_image(loaded, "examples/assets/skybox_evening/right.jpg").unwrap(),
                                 &Loader::get_image(loaded, "examples/assets/skybox_evening/left.jpg").unwrap(),
                                 &Loader::get_image(loaded, "examples/assets/skybox_evening/top.jpg").unwrap(),
                                 &Loader::get_image(loaded, "examples/assets/skybox_evening/front.jpg").unwrap(),
                                 &Loader::get_image(loaded, "examples/assets/skybox_evening/back.jpg").unwrap()).unwrap();

        let (penguin_cpu_meshes, penguin_cpu_materials) = Obj::parse(loaded, "examples/assets/PenguinBaseMesh.obj").unwrap();
        let penguin = renderer.new_meshes(&penguin_cpu_meshes, &penguin_cpu_materials).unwrap().remove(0);

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
                            println!("{:?}", renderer.debug_type());
                        }
                    }
                }
            }

            // draw
            // Geometry pass
            renderer.geometry_pass(width, height, &|| {
                let mut transformation = Mat4::identity();
                box_mesh.render_geometry(&transformation, &camera)?;
                transformation = Mat4::from_translation(vec3(-0.5, 1.0, 0.0));
                state::cull(&gl, state::CullType::Back);
                penguin.render_geometry(&transformation, &camera)?;
                Ok(())
            }).unwrap();

            renderer.render_to_screen_with_forward_pass(&camera, Some(&ambient_light), &[&directional_light], &[], &[], width, height, || {
                let transformation = Mat4::from_translation(vec3(0.5, 1.0, 0.0));
                state::cull(&gl, state::CullType::Back);
                penguin.mesh().render_with_ambient_and_directional(&transformation, &camera, &ambient_light, &directional_light)?;
                skybox.apply(&camera)?;
                Ok(())
            }).unwrap();

            #[cfg(target_arch = "x86_64")]
            if let Some(ref path) = screenshot_path {
                let pixels = Screen::read_color(&gl, 0, 0, width, height).unwrap();
                Saver::save_pixels(path, &pixels, width, height).unwrap();
                std::process::exit(1);
            }
        }).unwrap();
    });
}







fn main_old() {

    let mut window = Window::new_default("panorama_tool").unwrap();
    let (screen_width, screen_height) = window.framebuffer_size();
    let gl = window.gl();

    // Renderer
    let mut camera =
        Camera::new_perspective(&gl,
            vec3(0.0, 0.0, 5.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            screen_width as f32 / screen_height as f32, 0.1, 1000.0);


    let a = tri_mesh::MeshBuilder::new().square().build().unwrap();
    let mut a =
        Mesh::new(&gl,
            &a.indices_buffer(),
            &a.positions_buffer_f32(),
            &a.normals_buffer_f32()
        ).unwrap();

    a.texture = Some(
        texture::Texture2D::new_from_bytes(&gl,
           Interpolation::Linear,
           Interpolation::Linear,
           Some(Interpolation::Linear),
           Wrapping::ClampToEdge,
           Wrapping::ClampToEdge,
           include_bytes!("test_texture.jpg")
        ).unwrap());



    let mut renderer = DeferredPipeline::new(&gl).unwrap();
    let ambient_light =
        AmbientLight::new(&gl,
            1.0,
            &vec3(1.0, 1.0, 1.0)
        ).unwrap();
    let directional_light =
        DirectionalLight::new(&gl,
            0.0,
            &vec3(1.0, 1.0, 1.0),
            &vec3(0.0, -1.0, -1.0)
        ).unwrap();


    // main loop
    window.render_loop(move |frame_input|
    {
        camera.set_size(frame_input.screen_width as f32, frame_input.screen_height as f32);

        renderer.geometry_pass(screen_width, screen_height, &|| {
            let transformation = Mat4::identity();
            a.render(&transformation, &camera);
        }).unwrap();

        Screen::write(
            &gl, 0, 0, screen_width, screen_height,
            Some(&vec4(0.0, 0.0, 0.0, 1.0)), Some(1.0),
            &||{

                renderer.light_pass(&camera, Some(&ambient_light), &[&directional_light], &[], &[]).unwrap();

            }
        ).unwrap();

    }).unwrap();

}