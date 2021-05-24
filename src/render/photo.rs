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
}