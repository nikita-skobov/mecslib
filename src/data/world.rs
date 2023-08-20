use std::collections::HashMap;

use hecs::*;
use macroquad::prelude::*;

use crate::system::stateless::System;

use super::loading::TextureEnum;

pub trait UserState<T: TextureEnum>: Default {
    fn initialize(s: &mut State<Self, T>);
}

/// state that is managed by the application
#[derive(Default)]
pub struct State<U: UserState<T>, T: TextureEnum> {
    pub world: World,
    pub textures: HashMap<T, Texture2D>,
    /// user-defined state
    pub usr: U,
    pub clear_color: Color,
    pub coords: CoordTransform,
}

impl<U: UserState<T>, T: TextureEnum> State<U, T> {
    pub fn new() -> Self {
        let textures = T::load();
        let usr = U::default();
        let mut s = Self {
            usr,
            textures,
            clear_color: BLACK,
            coords: Default::default(),
            world: Default::default(),
        };
        U::initialize(&mut s);
        s
    }
}

/// main game loop. runs your game according to your
/// initial state data + the system functions you defined.
/// provide your initialized state+world,
/// a list of system functions to run in order,
/// and how many frames to output debug timings. if set to 0,
/// no debug timings are emitted.
pub async fn run<U: UserState<T>, T: TextureEnum>(
    mut state: State<U, T>,
    systems: &'static[System<U, T>],
    debug_frame_count: usize,
) {
    let frames_before_debug = debug_frame_count;
    let frames_before_debug_f64 = frames_before_debug as f64;
    let mut debug_frame_count = 0;
    let mut debug_timings: Vec<f64> = systems.iter().map(|_| 0.0).collect();
    let mut longest_sys_name = 0;
    for (_, sys_name) in systems.iter() {
        if sys_name.len() > longest_sys_name {
            longest_sys_name = sys_name.len();
        }
    }

    loop {
        clear_background(state.clear_color);
        let delta_time = get_frame_time();
        for (sys_index, (sys_fn, _sys_name)) in systems.iter().enumerate() {
            let start = macroquad::time::get_time();
            sys_fn(&mut state, delta_time);
            let end = macroquad::time::get_time();
            if frames_before_debug > 0 {
                debug_timings[sys_index] += (end - start) * 1000.0;
            }
        }

        debug_frame_count += 1;
        if debug_frame_count >= frames_before_debug && frames_before_debug > 0 {
            let total_time = debug_timings.iter().sum::<f64>() / frames_before_debug_f64;
            let mut debug_timings_sorted: Vec<_> = debug_timings.drain(..).enumerate().map(|(i, x)| (i, x)).collect();
            debug_timings_sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            for (index, time) in debug_timings_sorted {
                if time < 0.001 {
                    break;
                }
                let (_, sys_name) = systems[index];
                let avg_time = time / frames_before_debug_f64;
                let percent = (avg_time / total_time) * 100.0;
                // 25 bars max, so we divide by 4.0
                let num_bars = (percent / 4.0).max(1.0);
                let padding = " ".repeat(longest_sys_name - sys_name.len());

                let bars = "\u{2588}".repeat(num_bars as usize);
                macroquad::logging::warn!("{}{} {:0.4}ms {}", sys_name, padding, avg_time, bars);
            }
            macroquad::logging::warn!("");
            debug_timings = systems.iter().map(|_| 0.0).collect();
            debug_frame_count = 0;
        }
        next_frame().await;
    }
}
