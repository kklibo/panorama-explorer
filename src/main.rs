use three_d::window::{Window, WindowSettings};
use three_d::frame::FrameOutput;
use three_d::core::{Screen, ClearState};
use three_d::object::MeshProgram;
use three_d::io::Loader;
use three_d::camera::{Camera, CameraControl};
use three_d::math::vec3;

mod viewport_geometry;
mod read_pto;
mod photo;
mod control_state;
mod gui_controls;
mod render;
mod entities;

use viewport_geometry::{ViewportGeometry, WorldCoords};


fn main() {

    if cfg!(not(target_arch = "wasm32")) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    let window = Window::new(WindowSettings{title: "panorama_tool".to_string(), ..Default::default()}).unwrap();
    let context = window.gl().unwrap();

    let mut viewport_geometry = ViewportGeometry::try_new(
        WorldCoords{x:0.0, y:0.0},
        1_f64,
        12_u32,
        1_u32,
        15_u32,
        window.viewport().unwrap().width,
        window.viewport().unwrap().height,
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
    let reset_photos_string_file = "test_photos/reset_photos_string";
    let align_photos_string_file = "test_photos/align_photos_string";
    let map_overlay_image = "test_photos/map_test.jpg";
    let photo_images = vec!(
    //    "test_photos/test1_border.jpg",
    //    "test_photos/test2_border.jpg",
    //    "test_photos/test1.jpg",
    //    "test_photos/test2.jpg",
        "test_photos/DSC_9108_12_5.JPG",
        "test_photos/DSC_9109_12_5.JPG",
        "test_photos/DSC_9110_12_5.JPG",
    );

    //assemble filepaths to be loaded
    let filepaths =
    {
        let mut filepaths =
        vec!(
            pto_file,
            reset_photos_string_file,
            align_photos_string_file,
            map_overlay_image,
        );

        filepaths.append(&mut photo_images.clone());
        filepaths
    };

    Loader::load(&filepaths, move |loaded|
    {

        let mut entities = entities::Entities::new(
            &context,
            loaded,
            &pto_file,
            &reset_photos_string_file,
            &align_photos_string_file,
            map_overlay_image,
            &photo_images
        );

        let         texture_program = MeshProgram::new(&context, include_str!("shaders/texture.frag")).unwrap();
        let  texture_dewarp_program = MeshProgram::new(&context, include_str!("shaders/texture_dewarp.frag")).unwrap();
        let texture_dewarp2_program = MeshProgram::new(&context, include_str!("shaders/texture_dewarp2.frag")).unwrap();
        let           color_program = MeshProgram::new(&context, include_str!("shaders/color.frag")).unwrap();


        // main loop

        let mut control_state = control_state::ControlState::default();

        window.render_loop(move |mut frame_input|
        {
            viewport_geometry.set_pixel_dimensions(frame_input.viewport.width, frame_input.viewport.height).unwrap();

            let mut redraw = frame_input.first_frame;
            redraw |= camera.set_aspect(frame_input.viewport.aspect()).unwrap();
            camera.set_orthographic_projection(viewport_geometry.width_in_world_units() as f32,
                                               viewport_geometry.height_in_world_units() as f32,
                                               10.0).unwrap();

            redraw |= gui_controls::run_gui_controls(
                &mut frame_input,
                &mut gui,
                &mut control_state,
                &mut viewport_geometry,
                &mut entities
            );

            redraw |= gui_controls::handle_input_events(
                &mut frame_input,
                &mut control_state,
                &mut viewport_geometry,
                &mut camera,
                &mut entities.photos,
            );


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

                render::render(
                    &context,
                    &frame_input,
                    &mut gui,
                    &control_state,
                    &camera,
                    &viewport_geometry,
                    &texture_program,
                    &texture_dewarp_program,
                    &texture_dewarp2_program,
                    &color_program,
                    &entities,
                );


                //set entire display buffer alpha to 1.0: prevents web browser pass-through transparency problem
                let clear_alpha = ClearState {
                    red: None,
                    green: None,
                    blue: None,
                    alpha: Some(1.0),
                    depth: None,
                };
                Screen::write(&context, clear_alpha, || { Ok(()) }).unwrap();
            }

            FrameOutput {swap_buffers: redraw, ..Default::default()}

        }).unwrap();
    });
}
