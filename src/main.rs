
use three_d::*;

fn main() {

    let mut window = Window::new_default("panorama_tool").unwrap();
    let (screen_width, screen_height) = window.framebuffer_size();
    let gl = window.gl();

    // Renderer
    let mut camera =
        Camera::new_perspective(&gl,
            vec3(5.0, -3.0, 5.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 1.0, 0.0),
            degrees(45.0),
            screen_width as f32 / screen_height as f32, 0.1, 1000.0);

    // main loop
    window.render_loop(move |frame_input|
    {
        camera.set_size(frame_input.screen_width as f32, frame_input.screen_height as f32);

        Screen::write(
            &gl, 0, 0, screen_width, screen_height,
            Some(&vec4(0.0, 0.0, 0.0, 1.0)), Some(1.0),
            &||{}
        ).unwrap();

    }).unwrap();

}