use three_d::{Screen,ClearState,RenderStates,ColorTargetTexture2D};
use three_d::Error;

use crate::control_state::DewarpShader;
use super::Renderer;

impl Renderer<'_> {

    pub(in super) fn render_photos(&self, photo_alpha: f32, render_states: RenderStates) -> Result<(), Error> {

        for m in &self.entities.photos {
            let program = match self.control_state.dewarp_shader
            {
                DewarpShader::NoMorph => &self.texture_program,
                DewarpShader::Dewarp1 => &self.texture_dewarp_program,
                DewarpShader::Dewarp2 => &self.texture_dewarp2_program,
            };

            program.use_texture(&m.loaded_image_mesh.texture_2d, "tex").unwrap();
            program.use_uniform_float("out_alpha", &photo_alpha).unwrap();

            let mut mesh = m.loaded_image_mesh.mesh.clone();
            mesh.transformation = m.to_world();
            mesh.render(program, render_states, self.frame_input.viewport, &self.camera)?;
        }

        Ok(())
    }

    pub(in super) fn render_photos_with_pixel_averaging(&self) -> Result<(), Error> {

        use three_d::definition::{Interpolation, Wrapping, Format};

        let tmp_texture = ColorTargetTexture2D::<f32>::new(
            &self.context,
            self.frame_input.viewport.width,
            self.frame_input.viewport.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Format::RGBA,
        ).unwrap();

        let photo_texture = ColorTargetTexture2D::<u8>::new(
            &self.context,
            self.frame_input.viewport.width,
            self.frame_input.viewport.height,
            Interpolation::Nearest,
            Interpolation::Nearest,
            None,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Format::RGBA,
        ).unwrap();

        {
            tmp_texture.write(ClearState::color(0.0, 0.0, 0.0, 0.0), || {

                self.render_photos(1.0, Renderer::render_states_accumulate())

            }).unwrap();

            photo_texture.write(ClearState::none(), || {
                self.entities.average_effect.use_texture(&tmp_texture, "colorMap")?;
                self.entities.average_effect.apply(Renderer::render_states_no_blend(), self.frame_input.viewport)

            }).unwrap();
        }

        Screen::write(&self.context, ClearState::none(), || {

            self.entities.copy_photos_effect.use_texture(&photo_texture, "colorMap")?;
            self.entities.copy_photos_effect.apply(Renderer::render_states_transparency(), self.frame_input.viewport)

        })
    }
}