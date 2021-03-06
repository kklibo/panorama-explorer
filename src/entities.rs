use std::rc::Rc;

use three_d::{Loaded, Vec3, Context, ImageEffect, CullType};
use three_d::definition::{Interpolation, Wrapping};
use three_d::definition::CPUMesh;
use three_d::core::Texture2D;
use three_d::object::Mesh;

use log::info;

use crate::read_pto;
use crate::photo::Photo;
use crate::viewport_geometry::WorldCoords;


pub struct Entities {

    pub image0_control_points: Vec<Vec3>,
    pub image1_control_points: Vec<Vec3>,
    pub photos: Vec<Photo>,
    pub photos_alignment_string: String,
    pub photos_alignment_alt_string: String,
    pub color_mesh: Mesh,
    pub line_mesh: Mesh,
    pub overlay_mesh: Rc<LoadedImageMesh>,
    pub average_effect: ImageEffect,
    pub copy_photos_effect: ImageEffect,
}

impl Entities {

    pub fn new(
        context: &Context,
        loaded: &Loaded,
        pto_file: &str,
        photos_alignment_string_file: &str,
        photos_alignment_alt_string_file: &str,
        map_overlay_image: &str,
        photo_images: &Vec<&str>) -> Entities
    {
        let file_u8 = loaded.bytes(photos_alignment_string_file).unwrap();
        let photos_alignment_string = std::str::from_utf8(file_u8).unwrap().to_string();

        let file_u8 = loaded.bytes(photos_alignment_alt_string_file).unwrap();
        let photos_alignment_alt_string = std::str::from_utf8(file_u8).unwrap().to_string();

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

        for &Vec3 { x, y, z } in &image0_control_points {
            info!("({:?}, {:?}, {:?})", x, y, z);
        }

        let image1_control_points =
            pairs.iter().filter_map(|(_cp1, cp2)| {
                match cp2.image_id {
                    1 => Some(Vec3::new(cp2.x_coord as f32, cp2.y_coord as f32, 0 as f32)),
                    _ => None,
                }
            }).collect::<Vec<Vec3>>();

        let meshes: Vec<Rc<LoadedImageMesh>> = photo_images.iter().map(|x| {
            Rc::new(load_mesh_from_filepath(&context, loaded, x))
        }).collect();

        let mut photos: Vec<Photo> = meshes.iter().map(|mesh| {
            Photo::from_loaded_image_mesh(mesh.clone())
        }).collect();

        photos.get_mut(1).map(
            |photo| photo.set_translation(WorldCoords { x: 500.0, y: 0.0 })
        );

        photos.get_mut(2).map(
            |photo| photo.set_translation(WorldCoords { x: 1000.0, y: 0.0 })
        );

        let color_mesh = color_mesh(&context);
        let line_mesh = line_mesh(&context);
        let overlay_mesh = Rc::new(load_mesh_from_filepath(&context, loaded, map_overlay_image));

        let average_effect = ImageEffect::new(context, include_str!("shaders/average_effect.frag")).unwrap();
        let copy_photos_effect = ImageEffect::new(context, include_str!("shaders/copy_photos.frag")).unwrap();

        let mut entities = Entities{
            image0_control_points,
            image1_control_points,
            photos,
            photos_alignment_string,
            photos_alignment_alt_string,
            color_mesh,
            line_mesh,
            overlay_mesh,
            average_effect,
            copy_photos_effect,
        };

        entities.set_photos_from_json_serde_string(&entities.photos_alignment_string.clone()).unwrap();
        entities
    }

    pub fn set_photos_from_json_serde_string(&mut self, s: &str) -> Result<(), Box<dyn std::error::Error>> {

        for (index, line) in s.lines().enumerate() {

            if let Some(photo) = self.photos.get_mut(index) {
                photo.set_from_json_serde_string(line)?;
            };
        }
        Ok(())
    }

}


pub struct LoadedImageMesh {

    pub mesh: Mesh,
    pub texture_2d: Texture2D,
}

fn load_mesh_from_filepath(context: &Context, loaded: &Loaded, image_filepath: &str) -> LoadedImageMesh {

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

    let texture_2d = Texture2D::new(&context, &cpu_texture).unwrap();

    let mut mesh = Mesh::new(&context, &cpu_mesh).unwrap();
    mesh.cull = CullType::Back;

    LoadedImageMesh {mesh, texture_2d}
}

fn color_mesh(context: &Context) -> Mesh {

    let mut cpu_mesh = CPUMesh {
        positions: hourglass_positions(),

        ..Default::default()
    };
    cpu_mesh.compute_normals();

    let mut mesh = Mesh::new(&context, &cpu_mesh).unwrap();
    mesh.cull = CullType::Back;

    mesh
}

fn line_mesh(context: &Context) -> Mesh {

    let cpu_mesh = CPUMesh {
        positions: line_positions(),

        ..Default::default()
    };

    let mut mesh = Mesh::new(&context, &cpu_mesh).unwrap();
    mesh.cull = CullType::Back;

    mesh
}

pub fn square_positions() -> Vec<f32> {
    vec![
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
        0.5, 0.5, 0.0,
        0.5, 0.5, 0.0,
        -0.5, 0.5, 0.0,
        -0.5, -0.5, 0.0,
    ]
}

pub fn square_uvs() -> Vec<f32> {
    vec![
        0.0, 0.0,
        1.0, 0.0,
        1.0, 1.0,
        1.0, 1.0,
        0.0, 1.0,
        0.0, 0.0,
    ]
}

fn line_positions() -> Vec<f32> {
    vec![
        0.0, -0.5, 0.0,
        1.0, -0.5, 0.0,
        1.0, 0.5, 0.0,
        1.0, 0.5, 0.0,
        0.0, 0.5, 0.0,
        0.0, -0.5, 0.0,
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