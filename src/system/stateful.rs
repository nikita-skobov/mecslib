//! A stateful system is one that is tracked outside of the hecs world, ie:
//! in the UserState or GameState struct. These systems are useful to represent
//! singletons within the game world, such as but not limited to: maps, player characters, coordinate transforms, etc.

use macroquad::prelude::*;

pub struct CoordTransform {
    pub pan_x: f32,
    pub pan_y: f32,
    pub scale: f32,

    pub start_pan_x: f32,
    pub start_pan_y: f32,

    pub wasd_pan_by: f32,

    pub pan_keys_left: Vec<KeyCode>,
    pub pan_keys_right: Vec<KeyCode>,
    pub pan_keys_up: Vec<KeyCode>,
    pub pan_keys_down: Vec<KeyCode>,
    pub pan_mouse: Vec<MouseButton>,
    pub zoom_scroll_enabled: bool,
}
impl Default for CoordTransform {
    fn default() -> Self {
        Self {
            pan_x: Default::default(),
            pan_y: Default::default(),
            scale: 1.0,
            start_pan_x: Default::default(),
            start_pan_y: Default::default(),
            // TODO: make user editable setting (pan sensitivity)
            wasd_pan_by: 10.0,
            pan_keys_left: vec![KeyCode::Left],
            pan_keys_right: vec![KeyCode::Right],
            pan_keys_up: vec![KeyCode::Up],
            pan_keys_down: vec![KeyCode::Down],
            pan_mouse: vec![MouseButton::Left],
            zoom_scroll_enabled: true,
        }
    }
}

impl CoordTransform {
    pub const MAX_SCALE: f32 = 6.0;
    pub const MIN_SCALE: f32 = 0.1;
    pub const SCALE_BY: f32 = 0.05;

    pub fn to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x - self.pan_x) * self.scale,
            (y - self.pan_y) * self.scale,
        )
    }
    pub fn to_world(&self, x: f32, y: f32) -> (f32, f32) {
        (
            (x / self.scale) + self.pan_x,
            (y / self.scale) + self.pan_y,
        )
    }
    /// Center the view on a given point at the current scale
    pub fn center_to(&mut self, cx: f32, cy: f32) {
        // Calculate the screen center in world coordinates
        let (screen_width, screen_height) = (screen_width(), screen_height());

        // Calculate the pan values to center the view on the given point
        let new_pan_x = cx - screen_width / (2.0 * self.scale);
        let new_pan_y = cy - screen_height / (2.0 * self.scale);

        // Update the internal state with the new pan values
        self.pan_x = new_pan_x;
        self.pan_y = new_pan_y;
    }
}