use three_d::Vec3;
use crate::WorldCoords;


#[derive(PartialEq, Debug)]
pub enum DewarpShader {
    NoMorph,
    Dewarp1,
    Dewarp2,
}

pub struct Pan {
    pub mouse_start: (f64,f64),
    pub camera_start: Vec3,
}

pub struct Drag {
    pub mouse_start: (f64,f64),
    pub photo_start: WorldCoords,
    pub photo_index: usize, //replace this
}

pub struct RotationPoint {
    pub point: WorldCoords,
}

pub struct RotateDrag {
    pub mouse_start: WorldCoords,
    pub mouse_coords: WorldCoords,
    pub translate_start: WorldCoords,
    pub rotate_start: f32, //degrees
    pub photo_index: usize,
}

#[derive(Debug, PartialEq)]
pub enum MouseTool {
    PanView,
    DragPhoto,
    SelectPhoto,
    RotationPoint,
    DragToRotate,
}

pub struct ControlState {
    pub dewarp_shader: DewarpShader,
    pub active_pan: Option<Pan>,
    pub active_drag: Option<Drag>,
    pub active_rotation_point: Option<RotationPoint>,
    pub active_rotate_drag: Option<RotateDrag>,
    pub active_mouse_tool: MouseTool,

    pub selected_photo_index: Option<usize>,

    pub mouse_location_ui_text: String,
    pub photo_ui_text: String,
    pub control_points_visible: bool,

    pub alignment_mode: bool,
}

impl Default for ControlState {

    fn default() -> Self {
        Self {
            dewarp_shader: DewarpShader::Dewarp2,
            active_pan: None,
            active_drag: None,
            active_rotation_point: None,
            active_rotate_drag: None,
            active_mouse_tool: MouseTool::SelectPhoto,

            selected_photo_index: None,

            mouse_location_ui_text: "".to_string(),
            photo_ui_text: "".to_string(),
            control_points_visible: false,
            alignment_mode: false,
        }
    }
}