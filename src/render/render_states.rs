use three_d::{RenderStates,BlendParameters,BlendMultiplierType,BlendEquationType,WriteMask,DepthTestType};

pub fn render_states_transparency() -> RenderStates {
    RenderStates {
        blend: Some(BlendParameters {
            source_rgb_multiplier: BlendMultiplierType::SrcAlpha,
            source_alpha_multiplier: BlendMultiplierType::One,
            destination_rgb_multiplier: BlendMultiplierType::OneMinusSrcAlpha,
            destination_alpha_multiplier: BlendMultiplierType::Zero,
            rgb_equation: BlendEquationType::Add,
            alpha_equation: BlendEquationType::Add,
        }),

        write_mask: WriteMask::COLOR,
        depth_test: DepthTestType::Always,
    }
}

pub fn render_states_accumulate() -> RenderStates {
    RenderStates {
        blend: Some(BlendParameters {
            source_rgb_multiplier: BlendMultiplierType::One,
            source_alpha_multiplier: BlendMultiplierType::One,
            destination_rgb_multiplier: BlendMultiplierType::One,
            destination_alpha_multiplier: BlendMultiplierType::One,
            rgb_equation: BlendEquationType::Add,
            alpha_equation: BlendEquationType::Add,
        }),

        write_mask: WriteMask::COLOR,
        depth_test: DepthTestType::Always,
    }
}

pub fn render_states_no_blend() -> RenderStates {
    RenderStates {
        blend: None,
        write_mask: WriteMask::COLOR,
        depth_test: DepthTestType::Always,
    }
}
