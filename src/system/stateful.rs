//! A stateful system is one that is tracked outside of the hecs world, ie:
//! in the UserState or GameState struct. These systems are useful to represent
//! singletons within the game world, such as but not limited to: maps, player characters, coordinate transforms, etc.

use bracket_noise::prelude::*;
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

pub struct RandomMapGen {
    pub square_size: usize,
    pub noise: FastNoise,
    pub remaining: Vec<(i32, i32)>,
    pub fill_speed: usize,
}
impl Default for RandomMapGen {
    fn default() -> Self {
        Self {
            square_size: 0,
            fill_speed: 0,
            noise: FastNoise::new(),
            remaining: Default::default(),
        }
    }
}

impl RandomMapGen {
    pub fn new(square_size: usize, fill_speed: usize, seed: u64) -> Self {
        let mut noise = FastNoise::seeded(seed);
        noise.set_noise_type(NoiseType::PerlinFractal);
        noise.set_fractal_type(FractalType::FBM);
        noise.set_fractal_octaves(5);
        noise.set_fractal_gain(0.6);
        noise.set_fractal_lacunarity(2.0);
        noise.set_frequency(1.0);
        let mut remaining = Vec::with_capacity(square_size * square_size);
        for y in 0..square_size {
            for x in 0..square_size {
                remaining.push((x as _, y as _));
            }
        }
        Self {
            fill_speed,
            square_size,
            noise,
            remaining,
        }
    }
    pub fn set_noise(&mut self, mut cb: impl FnMut(&mut FastNoise)) {
        cb(&mut self.noise);
    }
    /// like get_next_n, but uses internal fill_speed as N.
    pub fn get_next(&mut self) -> Vec<(i32, i32, f32)> {
        self.get_next_n(self.fill_speed)
    }
    /// returns the next N tiles from the remaining list.
    /// if N is greater than the size of remaining list, returns everything left.
    pub fn get_next_n(&mut self, n: usize) -> Vec<(i32, i32 ,f32)> {
        let n = if n > self.remaining.len() {
            self.remaining.len()
        } else { n };
        let ratio_by = self.square_size as f32 * self.noise.get_frequency();
        let mut next = Vec::with_capacity(n);
        let mut arr = self.remaining.split_off(n);
        std::mem::swap(&mut arr, &mut self.remaining);
        for index in arr {
            let (x, y) = (
                index.0 as f32 / ratio_by,
                index.1 as f32 / ratio_by,
            );
            let height = self.noise.get_noise(x, y);
            next.push((index.0, index.1, height));
        }
        next
    }
    /// returns a vec of each tile with its coordinates, and the height of that tile.
    /// this uses up all of the remaining coordinates, so don't use if you've already called
    /// get_next_n
    pub fn finish(&mut self) -> Vec<(i32, i32, f32)> {
        self.get_next_n(self.remaining.len())
    }
}
