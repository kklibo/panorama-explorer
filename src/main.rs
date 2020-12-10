
use three_d::*;

fn main() {

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