/*
#[derive(PartialEq, Debug)]
enum DewarpShader {
    NoMorph,
    Dewarp1,
    Dewarp2,
}
let mut dewarp_shader = DewarpShader::NoMorph;

struct Pan {
    mouse_start: (f64,f64),
    camera_start: Vec3,
}
let mut active_pan: Option<Pan> = None;

struct Drag {
    mouse_start: (f64,f64),
    photo_start: WorldCoords,
    photo_index: usize, //replace this
}
let mut active_drag: Option<Drag> = None;

struct RotationPoint {
    point: WorldCoords,
    translate_start: WorldCoords,
    rotate_start: f32,
}
let mut active_rotation_point: Option<RotationPoint> = None;

struct RotateDrag {
    mouse_start: WorldCoords,
    mouse_coords: WorldCoords,
    rotate_start: f32, //degrees
    //photo_index: usize, //replace this
}
let mut active_rotate_drag: Option<RotateDrag> = None;

#[derive(Debug, PartialEq)]
enum MouseTool {
    RotationPoint,
    DragToRotate,
}
let mut active_mouse_tool: MouseTool = MouseTool::RotationPoint;

let mut dewarp_strength: f32 = 0.0;
let mut debug_rotation: f32 = 0.0;

let mut mouse_click_ui_text= "".to_string();
let mut photo_ui_text= "".to_string();
*/