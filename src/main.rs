
use three_d::*;

mod read_pto;

fn main() {

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
                                degrees(45.0),
                                width as f32 / height as f32, 0.1, 1000.0);

    Loader::load(&["src/test_texture.jpg"], move |loaded|
    {

        let mut square_cpu_mesh = CPUMesh {
            positions: square_positions(),
            ..Default::default()
        };
        square_cpu_mesh.compute_normals();
        let square_material = PhongMaterial {
            color_source: ColorSource::Texture(std::rc::Rc::new(texture::Texture2D::new_with_u8(&gl, Interpolation::Linear, Interpolation::Linear,
                                                                                                Some(Interpolation::Linear), Wrapping::Repeat, Wrapping::Repeat,
                                                                                                &Loader::get_image(loaded, "src/test_texture.jpg").unwrap()).unwrap())),
            ..Default::default()
        };
        let square_mesh = renderer.new_mesh(&square_cpu_mesh, &square_material).unwrap();


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
                let transformation = Mat4::identity();
                square_mesh.render_geometry(&transformation, &camera)?;
                Ok(())
            }).unwrap();

            renderer.render_to_screen_with_forward_pass(&camera, Some(&ambient_light), &[&directional_light], &[], &[], width, height, || {
                Ok(())
            }).unwrap();

        }).unwrap();
    });
}

fn square_positions() -> Vec<f32> {
    vec![
        -1.0, -1.0, 1.0,
        1.0, -1.0, 1.0,
        1.0, 1.0, 1.0,
        1.0, 1.0, 1.0,
        -1.0, 1.0, 1.0,
        -1.0, -1.0, 1.0,
    ]
}




/*
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
*/