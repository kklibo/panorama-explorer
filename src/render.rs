use three_d::MeshProgram;
use three_d::Screen;
use three_d::gui::GUI;
use three_d::{Context, CameraControl, FrameInput};

use crate::control_state::ControlState;
use crate::entities::Entities;
use crate::ViewportGeometry;

mod photo;
mod markers;
mod primitives;
mod render_states;
mod map_overlay;
mod colors;


/// Stores immutable references used in rendering
#[derive(Copy, Clone)]
pub struct Renderer<'a> {

    //three-d objects
    context: &'a Context,
    frame_input: &'a FrameInput,
    camera: &'a CameraControl,

    texture_program: &'a MeshProgram,
    texture_dewarp_program: &'a MeshProgram,
    texture_dewarp2_program: &'a MeshProgram,
    color_program: &'a MeshProgram,


    //crate objects
    viewport_geometry: &'a ViewportGeometry,
    control_state: &'a ControlState,
    entities: &'a Entities,
}


impl Renderer<'_> {

    pub fn new<'a>(
        context: &'a Context,
        frame_input: &'a FrameInput,
        camera: &'a CameraControl,

        texture_program: &'a MeshProgram,
        texture_dewarp_program: &'a MeshProgram,
        texture_dewarp2_program: &'a MeshProgram,
        color_program: &'a MeshProgram,

        viewport_geometry: &'a ViewportGeometry,
        control_state: &'a ControlState,
        entities: &'a Entities,
        ) -> Renderer<'a>
    {

        Renderer {
            context,
            frame_input,
            camera,

            texture_program,
            texture_dewarp_program,
            texture_dewarp2_program,
            color_program,

            viewport_geometry,
            control_state,
            entities,
        }
    }

    /// Use three-d to render the entire scene.
    ///
    /// Note: `gui` is passed as a separate parameter to allow mutability;
    /// all other needed references are in the Renderer object.
    pub fn render(&self, gui: &mut GUI)
    {
        Screen::write(&self.context, colors::main_window_clear(), || {

            //depth testing is not used: draw order determines visibility (last = most visible)

            //render photos
            if self.control_state.alignment_mode {

                //in alignment mode, use standard transparency
                self.render_photos(0.5, render_states::render_states_transparency())?;
            }
            else {

                //in browse mode, use pixel averaging
                self.render_photos_with_pixel_averaging()?;
            }

            if self.control_state.control_points_visible {

                //(temporary) point renderer hardcoded to the first 2 images
                self.render_control_points_temp()?;
            }

            if let Some(ref rp) = self.control_state.active_rotation_point {

                self.draw_point(rp.point, -45.0, colors::rotation_point())?;
            }

            if let Some(ref rp) = self.control_state.active_rotation_point {
                if let Some(ref rd) = self.control_state.active_rotate_drag {

                    self.draw_active_rotate_drag(rp, rd)?;
                }
            }

            if let Some(index) = self.control_state.selected_photo_index {

                if let Some(photo) = self.entities.photos.get(index) {
                    self.draw_selected_photo_border_rectangle(photo)?;
                }
            }

            //temporarily disabled
            //self.render_map_overlay();


            //render the egui UI
            gui.render()?;

            Ok(())

        }).unwrap();
    }
}