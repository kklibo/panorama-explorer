use three_d::{ClearState,ColorTargetTexture2D};
use three_d::Error;

use crate::control_state::{ControlState, DewarpShader};
use super::Renderer;

impl Renderer<'_> {

    pub fn render_photos(&self) -> Result<(), Error> {

        for m in &self.entities.photos {
            let program = match self.control_state.dewarp_shader
            {
                DewarpShader::NoMorph => &self.texture_program,
                DewarpShader::Dewarp1 => &self.texture_dewarp_program,
                DewarpShader::Dewarp2 => &self.texture_dewarp2_program,
            };

            program.use_texture(&m.loaded_image_mesh.texture_2d, "tex").unwrap();
            program.use_uniform_float("out_alpha", &0.5).unwrap();

            let mut mesh = m.loaded_image_mesh.mesh.clone();
            mesh.transformation = m.to_world();
            mesh.render(program, Renderer::render_states_transparency(),
                        self.frame_input.viewport, &self.camera)?;
        }

        Ok(())
    }

    pub fn render_photos_to_texture(&self) -> ColorTargetTexture2D<u8> {

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

        let out_texture = ColorTargetTexture2D::<u8>::new(
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

                for m in &self.entities.photos {
                    let program = match self.control_state.dewarp_shader
                    {
                        DewarpShader::NoMorph => &self.texture_program,
                        DewarpShader::Dewarp1 => &self.texture_dewarp_program,
                        DewarpShader::Dewarp2 => &self.texture_dewarp2_program,
                    };

                    program.use_texture(&m.loaded_image_mesh.texture_2d, "tex").unwrap();
                    program.use_uniform_float("out_alpha", &1.0).unwrap();

                    let mut mesh =  m.loaded_image_mesh.mesh.clone();
                    mesh.transformation = m.to_world();
                    mesh.render(program, Renderer::render_states_accumulate(), self.frame_input.viewport, &self.camera)?;
                }

                Ok(())
            }).unwrap();

            out_texture.write(ClearState::none(), || {
                self.entities.average_effect.use_texture(&tmp_texture, "colorMap")?;
                self.entities.average_effect.apply(Renderer::render_states_no_blend(), self.frame_input.viewport)

            }).unwrap();
        }

        out_texture
    }
}