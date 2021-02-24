
use three_d::*;
use log::info;

mod viewport_geometry;
mod read_pto;
mod photo;

use viewport_geometry::{ViewportGeometry, PixelCoords, WorldCoords};
use photo::{Photo, convert_photo_px_to_world};

pub struct LoadedImageMesh {

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

    let mut cpu_texture = Loader::get_texture(loaded, image_filepath).unwrap();
    cpu_texture.min_filter = Interpolation::Nearest;
    cpu_texture.mag_filter = Interpolation::Nearest;
    cpu_texture.mip_map_filter = None;
    cpu_texture.wrap_s = Wrapping::ClampToEdge;
    cpu_texture.wrap_t = Wrapping::ClampToEdge;
    cpu_texture.wrap_r = Wrapping::ClampToEdge;

    let texture = Texture2D::new_with_u8(&context, &cpu_texture).unwrap();

    let pixel_width = (&texture).width() as u32;
    let pixel_height =  (&texture).height() as u32;

    let material = PhongMaterial {
        color_source: ColorSource::Texture(std::rc::Rc::new(texture)),

        ..Default::default()
    };

    let mesh = PhongDeferredMesh::new(&context, &cpu_mesh, &material).unwrap();

    LoadedImageMesh {mesh, pixel_width, pixel_height}
}

fn color_mesh(context: &Context, color: &Vec4) -> PhongDeferredMesh {

    let mut cpu_mesh = CPUMesh {
        positions: square_positions(),

        ..Default::default()
    };
    cpu_mesh.compute_normals();

    let material = PhongMaterial {
        color_source: ColorSource::Color(color.clone()),

        ..Default::default()
    };

    let mesh = PhongDeferredMesh::new(&context, &cpu_mesh, &material).unwrap();

    mesh
}

fn main() {

    if cfg!(not(target_arch = "wasm32")) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    let window = Window::new("panorama_tool", None).unwrap();
    let context = window.gl();

    let mut viewport_geometry = ViewportGeometry::try_new(
        1_f64,
        10_u32,
        1_u32,
        15_u32,
        window.viewport().width,
        window.viewport().height,
    ).unwrap();


    // Renderer
    let mut pipeline = PhongDeferredPipeline::new(&context).unwrap();
    let mut camera =
        Camera::new_orthographic(&context,
                                 vec3(0.0, 0.0, 5.0),
                                 vec3(0.0, 0.0, 0.0),
                                 vec3(0.0, 1.0, 0.0),
                                 viewport_geometry.width_in_world_units() as f32,
                                 viewport_geometry.height_in_world_units() as f32,
                                 10.0);



    let pto_file = "test_photos/test.pto";

    let filepaths = [
        pto_file,
        "test_photos/test1.jpg",
        "test_photos/test2.jpg"
    ];

    Loader::load(&filepaths, move |loaded|
    {

        let file_u8 = Loader::get(loaded, pto_file).unwrap();
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

        let meshes = filepaths.iter().skip(1).map(|x| {
            load_mesh_from_filepath(&context, loaded, x)
        }).collect::<Vec<_>>();

        let orange_mesh = color_mesh(&context, &Vec4::new(0.8,0.5, 0.2, 1.0));
        let green_mesh = color_mesh(&context, &Vec4::new(0.2,0.8, 0.2, 1.0));

        let ambient_light = AmbientLight {intensity: 0.4, color: vec3(1.0, 1.0, 1.0)};
        let directional_light = DirectionalLight::new(&context, 1.0, &vec3(1.0, 1.0, 1.0), &vec3(0.0, -1.0, -1.0)).unwrap();

        // main loop

        struct Pan {
            mouse_start: (f64,f64),
            camera_start: Vec3,
        }
        let mut active_pan: Option<Pan> = None;

        window.render_loop(move |frame_input|
        {
            viewport_geometry.set_pixel_dimensions(frame_input.viewport.width, frame_input.viewport.height).unwrap();

            camera.set_aspect(frame_input.viewport.aspect());
            camera.set_orthographic_projection(viewport_geometry.width_in_world_units() as f32,
                                               viewport_geometry.height_in_world_units() as f32,
                                               10.0);

            for event in frame_input.events.iter() {
                match event {
                    Event::MouseClick {state, button, position} => {
                        info!("MouseClick: mouse position: {:?} {:?}", position.0, position.1);

                        active_pan =
                        match *button == MouseButton::Left && *state == State::Pressed {
                            true => Some(Pan {
                                mouse_start: *position,
                                camera_start: camera.position().clone(),
                            }),
                            false => None,
                        };

                    },
                    Event::MouseMotion {position, ..} => {

                        if let Some(ref mut pan) = active_pan {
                        //    info!("mouse delta: {:?} {:?}", delta.0, delta.1);
                        //    info!("mouse position: {:?} {:?}", position.0, position.1);

                            let camera_position_x = pan.camera_start.x - ((position.0 - pan.mouse_start.0) * viewport_geometry.world_units_per_pixel()) as f32;
                            let camera_position_y = pan.camera_start.y + ((position.1 - pan.mouse_start.1) * viewport_geometry.world_units_per_pixel()) as f32;

                            camera.set_view(
                                vec3(camera_position_x as f32, camera_position_y as f32, 5.0),
                                vec3(camera_position_x as f32, camera_position_y as f32, 0.0),
                                vec3(0.0, 1.0, 0.0)
                            );
                        }
                    },
                    Event::MouseWheel {delta, position} => {
                        info!("{:?}", delta);

                        let pixel_coords = PixelCoords{x: position.0, y: position.1};
                        let screen_coords = viewport_geometry.convert_pixel_to_screen(pixel_coords);

                        info!("cursor_screen {:?},{:?}", screen_coords.x, screen_coords.y);

                        //center the zoom action on the cursor
                        let to_cursor = viewport_geometry.convert_screen_to_world_at_origin(&screen_coords);
                        camera.translate(&Vec3::new(to_cursor.x as f32, to_cursor.y as f32, 0.0));

                        //un-reverse direction in web mode (not sure why it's backwards)
                        match (*delta > 0.0, cfg!(target_arch = "wasm32")) {
                            (true, true) | (false, false) => viewport_geometry.zoom_out(),
                            (true, false) | (false, true) => viewport_geometry.zoom_in(),
                        }

                        //and translate back, at the new zoom level
                        let to_cursor = viewport_geometry.convert_screen_to_world_at_origin(&screen_coords);
                        camera.translate(&Vec3::new(-to_cursor.x as f32, -to_cursor.y as f32, 0.0));

                        camera.set_orthographic_projection(viewport_geometry.width_in_world_units() as f32,
                                                           viewport_geometry.height_in_world_units() as f32,
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


            let mut photos = [
                Photo::from_loaded_image_mesh(&meshes[0]),
                Photo::from_loaded_image_mesh(&meshes[1]),
            ];
            photos[1].set_translation(WorldCoords{x: 500.0, y: 0.0});


            // draw
            // Geometry pass
            pipeline.geometry_pass(frame_input.viewport.width, frame_input.viewport.height, &|| {


                for m in &photos {

                    m.mesh.render_geometry(RenderStates {cull: CullType::Back, ..Default::default()},
                                                   frame_input.viewport, &m.to_world(), &camera)?;
                }

                let points = &image0_control_points;

                for &v in points {
                    let t1 = Mat4::from_nonuniform_scale(10.0,10.0,1.0);
                    let t1 = Mat4::from_translation(Vec3::new(0.0,0.0,1.0)).concat(&t1);

                    let t1 = convert_photo_px_to_world(v, &photos[0]).concat(&t1);
                    orange_mesh.render_geometry(RenderStates {cull: CullType::None, ..Default::default()},
                                               frame_input.viewport, &t1, &camera)?;
                }

                let points = &image1_control_points;

                for &v in points {
                    let t1 = Mat4::from_nonuniform_scale(10.0,10.0,1.0);
                    let t1 = Mat4::from_angle_z(cgmath::Deg(45.0)).concat(&t1);
                    let t1 = Mat4::from_translation(Vec3::new(0.0,0.0,1.0)).concat(&t1);

                    let t1 = convert_photo_px_to_world(v, &photos[1]).concat(&t1);
                    green_mesh.render_geometry(RenderStates {cull: CullType::None, ..Default::default()},
                                               frame_input.viewport, &t1, &camera)?;
                }

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
