use std::rc::Rc;

use three_d::window::Window;
use three_d::context::Context;
use three_d::definition::cpu_texture::{Interpolation, Wrapping};
use three_d::definition::cpu_mesh::CPUMesh;
use three_d::frame::output::FrameOutput;
use three_d::core::render_target::{Screen, ClearState};
use three_d::core::texture::Texture2D;
use three_d::object::{Mesh, MeshProgram};
use three_d::io::{Loader, Loaded};
use three_d::camera::{Camera, CameraControl};
use three_d::math::{Vec3, vec3};

use log::info;

mod viewport_geometry;
mod read_pto;
mod photo;
mod control_state;
mod gui_controls;
mod render;
mod entities;

use viewport_geometry::{ViewportGeometry, WorldCoords};
use photo::Photo;

//
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
//

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
//
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
//
        let         texture_program = MeshProgram::new(&context, include_str!(        "texture.frag")).unwrap();
        let  texture_dewarp_program = MeshProgram::new(&context, include_str!( "texture_dewarp.frag")).unwrap();
        let texture_dewarp2_program = MeshProgram::new(&context, include_str!("texture_dewarp2.frag")).unwrap();
        let           color_program = MeshProgram::new(&context, include_str!(          "color.frag")).unwrap();


        // main loop

        let mut control_state = control_state::ControlState::default();

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

            redraw |= gui_controls::run_gui_controls(&mut frame_input, &mut gui, &mut control_state);

            redraw |= gui_controls::handle_input_events(
                &mut frame_input,
                &mut control_state,
                &mut viewport_geometry,
                &mut camera,
                &mut photos,
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
                    &mut frame_input,
                    &mut gui,
                    &mut control_state,
                    &mut camera,
                    &photos,
                    &texture_program,
                    &texture_dewarp_program,
                    &texture_dewarp2_program,
                    &color_program,
                    &color_mesh,
                    &image0_control_points,
                    &image1_control_points
                );


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

//
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
//