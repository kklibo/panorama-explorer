use three_d::{Vec4, ClearState};

pub fn main_window_clear() -> ClearState {
    ClearState::color_and_depth(0.2, 0.2, 0.2, 1.0, 1.0)
}

#[allow(dead_code)]
pub fn map_overlay_clear() -> ClearState {
    ClearState::color(0.0, 0.5, 0.0, 0.0)
}

pub fn photo1_control_points_temp() -> Vec4 {
    Vec4::new(0.8, 0.5, 0.2, 0.5)
}

pub fn photo2_control_points_temp() -> Vec4 {
    Vec4::new(0.2, 0.8, 0.2, 0.5)
}

pub fn rotation_point() -> Vec4 {
    Vec4::new(0.8, 0.8, 0.2, 0.5)
}

pub fn dragged_rotation_triangle() -> Vec4 {
    Vec4::new(0.2, 0.2, 0.8, 0.5)
}

pub fn dragged_rotation_angle_lines() -> Vec4 {
    Vec4::new(0.8, 0.8, 0.2, 1.0)
}

pub fn selected_photo_border_rectangle() -> Vec4 {
    Vec4::new(0.2, 0.8, 0.2, 1.0)
}