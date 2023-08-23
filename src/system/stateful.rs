//! A stateful system is one that is tracked outside of the hecs world, ie:
//! in the UserState or GameState struct. These systems are useful to represent
//! singletons within the game world, such as but not limited to: maps, player characters, coordinate transforms, etc.

use std::collections::HashSet;

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
    pub const MAX_SCALE: f32 = 10.0;
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

#[derive(Default)]
pub struct RecursiveTiling {
    /// the algorithm will continue to subdivide
    /// tiles until every tile is <= desired_tile_size
    pub desired_tile_size: usize,
    pub ready_to_tile: bool,
    pub size: i32,
    pub open_set: HashSet<(i32, i32)>,
    pub past_sets: Vec<HashSet<(i32, i32)>>,
    pub current_a_set: HashSet<(i32, i32)>,
    pub current_b_set: HashSet<(i32, i32)>,
    pub a_done: HashSet<(i32, i32)>,
    pub current_a_frontier: Vec<(i32, i32)>,
    pub current_b_frontier: Vec<(i32, i32)>,
    pub current_a_frontier_set: HashSet<(i32, i32)>,
    pub current_b_frontier_set: HashSet<(i32, i32)>,
    pub all_done: bool,
    pub should_reset_animation: bool,
}
impl RecursiveTiling {
    pub fn should_reset(&mut self) -> bool {
        if self.should_reset_animation {
            self.should_reset_animation = false;
            return true
        }
        false
    }
    pub fn remove_two_random_from_set(set: &mut HashSet<(i32, i32)>, rng: &mut fastrand::Rng, size: i32) -> Option<[(i32, i32); 2]> {
        if set.len() < 2 { return None }

        // if there is less than 50% remaining, just put them into a vec, sort
        // and pick randomly from the vec
        let max_size = (size * size) as usize;
        let pct_remaining = set.len() as f32 / max_size as f32;
        if pct_remaining <= 0.5 {
            let mut total_set = Vec::with_capacity(set.len());
            for i in set.iter() {
                total_set.push(*i);
            }
            total_set.sort();
            let rand_index = rng.usize(0..total_set.len());
            let rand_a = total_set.swap_remove(rand_index);
            let rand_index = rng.usize(0..total_set.len());
            let rand_b = total_set.swap_remove(rand_index);
            set.remove(&rand_a);
            set.remove(&rand_b);
            return Some([rand_a, rand_b])
        }

        let mut first = None;
        loop {
            let rand_x = rng.i32(0..size);
            let rand_y = rng.i32(0..size);
            let rand_pair = (rand_x, rand_y);
            if set.contains(&rand_pair) {
                set.remove(&rand_pair);
                if let Some(f) = first {
                    return Some([f, rand_pair]);
                } else {
                    first = Some(rand_pair);
                }
            }
        }
    }
    pub fn get_random_from_frontier(set: &HashSet<(i32, i32)>) -> Option<(i32, i32)> {
        for i in set.iter() {
            return Some(*i);
        }
        return None
    }

    pub fn get_surrounding(i: impl Into<Option<(i32, i32)>>, include_self: bool) -> Vec<(i32, i32)> {
        let opt: Option<(i32, i32)> = i.into();
        let (x, y) = match opt {
            Some(a) => a,
            None => return vec![],
        };
        let mut out = vec![
            (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
            (x - 1, y),                 (x + 1, y    ),
            (x - 1, y + 1), (x, y + 1), (x + 1, y + 1),
        ];
        if include_self {
            out.push((x, y));
        }
        out
    }
    /// calls next N times. returns a vec that contains the output of all N calls.
    pub fn next_n(&mut self, rng: &mut fastrand::Rng, n: usize) -> (bool, Vec<(i32, i32)>, Vec<(i32, i32)>) {
        let mut a_new = vec![];
        let mut b_new = vec![];
        let mut should_reset = false;
        for _ in 0..n {
            let (reset, a, b) = self.next(rng);
            a_new.extend(a);
            b_new.extend(b);
            if reset {
                should_reset = true;
                break;
            }
        }
        (should_reset, a_new, b_new)
    }
    pub fn check_frontier(
        frontier: &mut Vec<(i32, i32)>,
        frontier_set: &mut HashSet<(i32, i32)>,
        open_set: &mut HashSet<(i32, i32)>,
        current_set: &mut HashSet<(i32, i32)>,
        current_other_set: &mut HashSet<(i32, i32)>,
        pushed: &mut Vec<(i32, i32)>,
        _rng: &mut fastrand::Rng,
    ) {
        // random doesnt work?
        // let max = 2.min(frontier.len());
        // let random = rng.usize(0..=max);
        // get and remove a random point from the current A frontier
        let random = 0;
        // drain frontier and handle everything currently in it
        let next_a = match frontier.get(random) {
            Some(a) => *a,
            None => return,
        };
        frontier.remove(0);
        frontier_set.remove(&next_a);
        // next_a is already in current_set.
        // visit its surrounding neighbors, add them to the frontier, and
        // add them to A
        for pt in Self::get_surrounding(next_a, false) {
            // dont check point again if its already in our set
            if current_set.contains(&pt) { continue; }
            // dont check point if its in the enemy set
            if current_other_set.contains(&pt) { continue;}
            // dont check point if it doesnt exist in the map
            if !open_set.contains(&pt) { continue; }
            if !frontier_set.contains(&pt) {
                frontier.push(pt);
                frontier_set.insert(pt);
            }
            // associate it with our current set
            current_set.insert(pt);
            open_set.remove(&pt);
            pushed.push(pt);
        }
    }
    pub fn get_next_set_to_divide(&mut self) -> Option<HashSet<(i32, i32)>> {
        let mut found = None;
        for (i, set) in self.past_sets.iter().enumerate() {
            if set.len() > self.desired_tile_size {
                found = Some(i);
                break;
            }
        }
        let set = match found {
            Some(i) => self.past_sets.remove(i),
            None => return None,
        };
        Some(set)
    }
    /// returns 2 vectors and a bool: whether or not to reset the animation,
    /// new tiles inserted into the current a set, and the b set.
    pub fn next(&mut self, rng: &mut fastrand::Rng) -> (bool, Vec<(i32, i32)>, Vec<(i32, i32)>) {
        let mut a_new = vec![];
        let mut b_new = vec![];
        let mut should_reset = false;
        if self.all_done {
            return (true, a_new, b_new)
        }
        if self.current_a_frontier.is_empty() && self.current_b_frontier.is_empty() {
            // our current open set can still be subdivided, so keep subdividing
            if self.open_set.len() > 1 {
                if let Some(random_pts) = Self::remove_two_random_from_set(&mut self.open_set, rng, self.size) {
                    self.current_a_frontier.push(random_pts[0]);
                    self.current_b_frontier.push(random_pts[1]);
                    self.current_a_frontier_set.insert(random_pts[0]);
                    self.current_b_frontier_set.insert(random_pts[1]);
                    self.open_set.remove(&random_pts[0]);
                    self.open_set.remove(&random_pts[1]);
                    a_new.push(random_pts[0]);
                    b_new.push(random_pts[1]);
                }
                for a in a_new.iter() {
                    self.current_a_set.insert(*a);
                }
                for b in b_new.iter() {
                    self.current_b_set.insert(*b);
                }
            } else {
                // current open set is empty (or size=1)
                // recurse, and now look at a past set that we've split
                // (if it's larger than the desired set size)
                if let Some(next_set) = self.get_next_set_to_divide() {
                    self.open_set = next_set;
                    self.should_reset_animation = true;
                    should_reset = true;
                } else {
                    self.should_reset_animation = true;
                    // DONE: all sets in our history
                    // are <= our desired tile size
                    self.all_done = true;
                }
            }
            return (should_reset, a_new, b_new)
        }

        // check and advance frontier for A first, then B.
        Self::check_frontier(
            &mut self.current_a_frontier,
            &mut self.current_a_frontier_set,
            &mut self.open_set,
            &mut self.current_a_set,
            &mut self.current_b_set,
            &mut a_new,
            rng,
        );
        Self::check_frontier(
            &mut self.current_b_frontier,
            &mut self.current_b_frontier_set,
            &mut self.open_set,
            &mut self.current_b_set,
            &mut self.current_a_set,
            &mut b_new,
            rng,
        );
        // we filled these 2 sets as much as we could.
        // add them to our set history, and reset state.
        if self.current_b_frontier.is_empty() && self.current_a_frontier.is_empty() {
            let a_set = std::mem::take(&mut self.current_a_set);
            let b_set = std::mem::take(&mut self.current_b_set);
            self.past_sets.push(a_set);
            self.past_sets.push(b_set);
        }

        (should_reset, a_new, b_new)
    }
}

#[derive(Default)]
pub struct VoronoiTiling {
    pub desired_points: usize,
    pub original_open_set_size: usize,
    pub open_set: HashSet<(i32, i32)>,
    pub open_set_list: Vec<(i32, i32)>,
    pub growth_starts: Vec<(i32, i32)>,
    pub growth_frontiers: Vec<Vec<(i32, i32)>>,
    pub growth_frontier_sets: Vec<HashSet<(i32, i32)>>,
    pub growth_sets: Vec<HashSet<(i32, i32)>>,
    pub ready_to_tile: bool,
    pub current_radius: f32,
    pub done: bool,
    pub grid_points: Option<(usize, i32, f32)>,
}
impl VoronoiTiling {
    const GROWTH_TOLERANCE: f32 = std::f32::consts::SQRT_2 - 1.0;

    /// instead of using random points from the open set,
    /// creates a grid of points of specified density (how far apart each point should be)
    /// random_intensity controls how much of an offset each grid point can have.
    /// eg: 0.0 => no offset, 10.0 => implies each point is shifted 10.0 points in a random direction
    pub fn with_grid_points(&mut self, grid_size: usize, density: i32, random_intensity: f32) {
        self.grid_points = Some((grid_size, density, random_intensity));
    }
    /// resets all fields except the open set.
    /// re-calculates desired_points to be scaled down
    /// depending on how much is remaining.
    /// do not call if open set is empty
    pub fn continue_with_open_set(&mut self) {
        if self.open_set.is_empty() {
            // you shouldnt be calling this if open_set is empty, so exit early :shrug:
            return;
        }
        self.done = false;
        self.current_radius = 0.0;
        self.growth_frontier_sets.clear();
        self.growth_frontiers.clear();
        self.growth_sets.clear();
        self.growth_starts.clear();
        self.open_set_list.clear();
        self.grid_points = None;
        for pt in self.open_set.iter() {
            self.open_set_list.push(*pt);
        }

        let original_ratio = self.desired_points as f32 / self.original_open_set_size as f32;
        self.desired_points = (self.open_set.len() as f32 * original_ratio) as usize;
        // we must ensure desired points is >= 1
        // but also less than the size of the open set list
        self.desired_points = self.desired_points.clamp(1, self.open_set.len());
    }
    /// calls next N times. returns a vec that contains the output of all N calls.
    pub fn next_n(&mut self, rng: &mut fastrand::Rng, n: usize) -> Vec<Vec<(i32, i32)>> {
        let mut next_data = vec![];
        for _ in 0..n {
            let mut data = self.next(rng);
            if next_data.is_empty() {
                next_data.extend(data);
            } else {
                for (i, d) in data.drain(..).enumerate() {
                    next_data[i].extend(d);
                }
            }
        }
        next_data
    }
    pub fn get_surrounding(i: (i32, i32)) -> [(i32, i32); 8] {
        let (x, y) = i;
        [
            (x - 1, y - 1), (x, y - 1), (x + 1, y - 1),
            (x - 1, y),                 (x + 1, y    ),
            (x - 1, y + 1), (x, y + 1), (x + 1, y + 1),
        ]
    }
    /// returns true if successfully initialized with grid points
    pub fn initialize_with_grid_points(&mut self, out: &mut Vec<Vec<(i32, i32)>>, rng: &mut fastrand::Rng) -> bool {
        let (grid_size, density, rng_intensity) = if let Some(x) = self.grid_points {
            x
        } else { return false };

        // create a grid of points within the open set.
        let mut grid = HashSet::new();
        let mut y = 0;
        let grid_size_i32 = grid_size as i32;
        while y < grid_size_i32 {
            let mut x = 0;
            while x < grid_size_i32 {
                if self.open_set.contains(&(x, y)) {
                    grid.insert((x, y));
                }
                x += density;
            }
            y += density;
        }
        // if theres rng intensity, apply it to each point:
        if rng_intensity > 0.0 {
            let mut new_grid = HashSet::new();
            let mut grid_vec: Vec<_> = grid.drain().collect();
            grid_vec.sort();
            for pt in grid_vec {
                let pt = Vec2::new(pt.0 as f32, pt.1 as f32);
                let rand = rng.f32();
                let rand_angle = rand * std::f32::consts::PI * 2.0;
                let rand_vec = Vec2::from_angle(rand_angle);
                let pt = pt + (rand_vec * rng_intensity);
                let (x, y) = (pt.x as i32, pt.y as i32);
                if self.open_set.contains(&(x, y)) {
                    new_grid.insert((x, y));
                }
            }
            grid = new_grid;
        }

        self.desired_points = grid.len();
        let mut random_pts = Vec::with_capacity(self.desired_points);
        for pt in grid {
            self.open_set.remove(&pt);
            random_pts.push(pt);
            out.push(vec![pt]);
            let mut set = HashSet::new();
            set.insert(pt);
            self.growth_frontiers.push(vec![pt]);
            let mut frontier_set = HashSet::new();
            frontier_set.insert(pt);
            self.growth_frontier_sets.push(frontier_set);
            self.growth_sets.push(set);
        }
        self.current_radius = 1.0;
        self.growth_starts = random_pts;
        true
    }
    /// returns 2 vectors and a bool: whether or not to reset the animation,
    /// new tiles inserted into the current a set, and the b set.
    pub fn next(&mut self, rng: &mut fastrand::Rng) -> Vec<Vec<(i32, i32)>> {
        let mut out = vec![];
        // first iteration: calculate all the random points
        if self.growth_starts.is_empty() {
            self.original_open_set_size = self.open_set.len();
            if self.initialize_with_grid_points(&mut out, rng) {
                return out;
            }
            // invalid state: cant start the algorithm if theres not enough data
            if self.open_set_list.len() < self.desired_points { return out; }
            let mut random_pts = Vec::with_capacity(self.desired_points);
            for _ in 0..self.desired_points {
                let random_i = rng.usize(0..self.open_set_list.len());
                let pt = self.open_set_list.swap_remove(random_i);
                self.open_set.remove(&pt);
                random_pts.push(pt);
                out.push(vec![pt]);
                let mut set = HashSet::new();
                set.insert(pt);
                self.growth_frontiers.push(vec![pt]);
                let mut frontier_set = HashSet::new();
                frontier_set.insert(pt);
                self.growth_frontier_sets.push(frontier_set);
                self.growth_sets.push(set);
            }
            self.current_radius = 1.0;
            self.growth_starts = random_pts;
            return out;
        }

        let mut all_frontier_sets_empty = true;
        for (i, growth_start) in self.growth_starts.iter().enumerate() {
            let mut claimed_for_i = vec![];
            let growth_start = Vec2::new(growth_start.0 as f32, growth_start.1 as f32);
            // get the frontier for this growth
            // let frontier = &mut self.growth_frontiers[i];
            let mut new_frontier_set = HashSet::new();
            let frontier_set = &self.growth_frontier_sets[i];
            let claimed_set = &mut self.growth_sets[i];
            let frontier = std::mem::take(&mut self.growth_frontiers[i]);
            let new_frontier = &mut self.growth_frontiers[i];
            for f_pt in frontier {
                // get all neighbors of this point.
                for neighbor_pt in Self::get_surrounding(f_pt) {
                    // exclude ones that are already in our frontier
                    // exclude ones that are already in our set.
                    if frontier_set.contains(&neighbor_pt) { continue; }
                    if claimed_set.contains(&neighbor_pt) { continue; }
                    // check if they are within the current radius.
                    let pt = Vec2::new(neighbor_pt.0 as f32, neighbor_pt.1 as f32);
                    let dist = pt.distance(growth_start);
                    if dist > self.current_radius + Self::GROWTH_TOLERANCE { continue; }
                    // check they are still in the open set
                    // and assign to this growth.
                    if !self.open_set.contains(&neighbor_pt) { continue; }
                    self.open_set.remove(&neighbor_pt);
                    claimed_set.insert(neighbor_pt);
                    new_frontier.push(neighbor_pt);
                    new_frontier_set.insert(neighbor_pt);
                    claimed_for_i.push(neighbor_pt);
                }
            }
            out.push(claimed_for_i);
            if !new_frontier_set.is_empty() {
                all_frontier_sets_empty = false;
            }
            self.growth_frontier_sets[i] = new_frontier_set;
        }
        self.current_radius += 1.0;
        if all_frontier_sets_empty {
            self.done = true;
        }
        out
    }
}
