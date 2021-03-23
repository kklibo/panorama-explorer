
pub struct Entities {

}

impl Entities {

/*
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
*/

}

/*
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
*/

/*
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
*/