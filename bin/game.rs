use std::collections::{HashSet, HashMap};

use hecs::*;
use macroquad::prelude::*;
use mecslib::{
    components::*,
    system::{stateless::*, stateful::*},
    data::{
        loading::TextureEnum,
        world::{State, UserState, run},
    },
    create_texture_enum,
    sys,
};

pub struct BuildingMapTile;

#[derive(Default)]
pub struct FilledMap {
    pub data: Vec<Vec<Color>>,
}


pub struct MyState {
    pub rand_map: RandomMapGen,
    pub rng: fastrand::Rng,
    pub filled: FilledMap,
    pub voronoi_tiling: VoronoiTiling,
    pub created_tile_map: bool,
    pub voronoi_colors: Vec<Color>,
    pub tile_positions: HashMap<(i32, i32), Entity>,
}

impl Default for MyState {
    fn default() -> Self {
        Self {
            rng: fastrand::Rng::with_seed(7),
            rand_map: Default::default(),
            filled: Default::default(),
            created_tile_map: Default::default(),
            voronoi_tiling: Default::default(),
            voronoi_colors: Default::default(),
            tile_positions: Default::default(),
        }
    }
}

impl UserState<Textures> for MyState {
    fn initialize(s: &mut State<Self, Textures>) {
        let grid_size = 1000;
        s.usr.rand_map = RandomMapGen::new(grid_size, 40000, s.usr.rng.u64(0..u64::MAX));
        // s.usr.voronoi_tiling.desired_points = 210;
        let density = grid_size as f32 * 0.038;
        let intensity = density / 2.0;
        s.usr.voronoi_tiling.with_grid_points(grid_size, density as _, intensity);

        // example of parenting:
        // let transform = Transform::from_scale_angle_position(1.0, 0.0, (0.0, 0.0));
        // let draw = Drawable::texture(s, Textures::test);
        // let unit = s.world.spawn((transform, draw, Layer1));

        // let transform = Transform::from_scale_angle_position(1.0, std::f32::consts::FRAC_PI_4, (10.0, 0.0));
        // let draw = Drawable::texture(s, Textures::other);
        // let parent = Parent {
        //     parent: unit,
        //     local_transform: transform,
        // };
        // let _other = s.world.spawn((Transform::default(), draw, parent, Layer2));
    }
}

pub type MySystem = System<MyState, Textures>;
pub type GameState = State<MyState, Textures>;

create_texture_enum!(Textures; other, test, empty);

pub struct IsTile;
const WATER_COLOR: Color = BLUE;
const NON_HOVERED_TILE_COLOR: Color = Color::new(0.7, 0.7, 0.7, 1.0);
const HOVERED_TILE_COLOR: Color = WHITE;

fn get_all_systems() -> &'static [MySystem] {
    &[
        sys!(handle_pan),
        sys!(update_children_transforms),
        sys!(generate_tiles_voronoi),
        sys!(draw_hovered_tiles),
        sys!(fill_generated_map),
        sys!(draw),
    ]
}

pub fn create_rivers(s: &mut GameState) {
    const MAX_COST: i32 = 3;
    const MAX_RIVERS: usize = 3;
    // generate a random number of rivers:
    let rand_num = s.usr.rng.usize(1..=MAX_RIVERS);
    let mut obstacles = HashSet::new();
    let grid_size = s.usr.rand_map.square_size as f32;

    // generate a random cost field so that
    // the river naturally curves around invisible "obstacles"
    let mut cost_map = HashMap::new();
    for y in 0..s.usr.rand_map.square_size {
        for x in 0..s.usr.rand_map.square_size {
            let rand_cost = s.usr.rng.i32(1..MAX_COST);
            cost_map.insert((x as i32, y as i32), rand_cost);
        }
    }

    for _ in 0..rand_num {
        // generate a random point inside the voronoi set of land tiles:
        let rand_u = s.usr.rng.usize(0..s.usr.voronoi_tiling.open_set_list.len());
        let start = s.usr.voronoi_tiling.open_set_list[rand_u];
        let start_vec = Vec2::new(start.0 as f32, start.1 as f32);
        // pick the furthest possible corner to try to expand to.
        let goal_opt1 = Vec2::new(0.0, 0.0);
        let goal_opt2 = Vec2::new(grid_size, 0.0);
        let goal_opt3 = Vec2::new(grid_size, grid_size);
        let goal_opt4 = Vec2::new(0.0, grid_size);
        let mut max_goal_dist = f32::MIN;
        let mut max_goal_pt = (0, 0);
        for goal_opt in [goal_opt1, goal_opt2, goal_opt3, goal_opt4] {
            let dist = goal_opt.distance(start_vec);
            if dist > max_goal_dist {
                max_goal_dist = dist;
                max_goal_pt = (goal_opt.x as i32, goal_opt.y as i32);
            }
        }
        let goal = max_goal_pt;
        let mut astar = Astar::new(start, goal);
        astar.path_mode = AstarPathMode::LateralOnly;
        astar.set_impassable(obstacles.clone());
        astar.set_costs(cost_map.clone());
        let path = if let Some(p) = astar.calculate_path() {
            p
        } else { continue; };
        // color every tile along the path blue for water.
        let mut thick_river_set = HashSet::new();
        for (x, y) in path {
            // if this point is in the ocean (ie: not in the voronoi open set => water)
            // then lets stop advancing the river. we dont want it to flow
            // through the ocean, as it risks cutting through islands in weird ways.
            if !s.usr.voronoi_tiling.open_set.contains(&(x, y)) {
                break;
            }
            // add this point to the river set:
            thick_river_set.insert((x, y));
            // as well as every point around it:
            // this is what makes the river "thick"
            for (x, y) in VoronoiTiling::get_surrounding((x, y)) {
                thick_river_set.insert((x, y));
            }
        }
        let mut thick_river_vec = Vec::with_capacity(thick_river_set.len());
        for (x, y) in thick_river_set.drain() {
            let remove_pt = (x, y);
            let x = x as usize;
            let y = y as usize;
            // lookup the grid tile at these indices
            let row = if let Some(r) = s.usr.filled.data.get_mut(y) {
                r
            } else { continue };
            let value = if let Some(v) = row.get_mut(x) {
                v
            } else { continue };
            s.usr.voronoi_tiling.open_set.remove(&remove_pt);
            thick_river_vec.push(remove_pt);
            // add the river as an obstacle so future rivers dont try to cross it.
            // but do not add it as an obstacle if its already in the water (eg: ocean)
            // otherwise future rivers will never be able to reach the goal.
            if *value != WATER_COLOR {
                obstacles.insert(remove_pt);
            }
            *value = WATER_COLOR;
        }
        thick_river_vec.sort();
        // remove everything from the voronoi open_set_list
        for pt in thick_river_vec {
            if let Some(i) = s.usr.voronoi_tiling.open_set_list.iter().position(|x| *x == pt) {
                s.usr.voronoi_tiling.open_set_list.swap_remove(i);
            }
        }
    }
}

fn draw_hovered_tiles(s: &mut GameState, _dt: f32) {
    // for each invocation make sure we hide tiles that are not hovered:
    let mut cb = CommandBuffer::new();
    for (entity, _) in s.world.query_mut::<&IsTile>() {
        cb.insert_one(entity, Tint { d: NON_HOVERED_TILE_COLOR });
    }
    cb.run_on(&mut s.world);

    // then unhide the tiles we hover:
    let (mx, my) = mouse_position();
    let (wx, wy) = s.coords.to_world(mx, my);
    let i32_coord = (wx as i32, wy as i32);
    let entity = match s.usr.tile_positions.get(&i32_coord) {
        Some(entity) => *entity,
        None => return,
    };
    let _ = s.world.insert_one(entity, Tint { d: HOVERED_TILE_COLOR });
}

fn generate_tiles_voronoi(s: &mut GameState, _dt: f32) {
    let tiling = &mut s.usr.voronoi_tiling;
    if !tiling.ready_to_tile || tiling.done {
        return;
    }
    let mut growths = tiling.next_n(&mut s.usr.rng, 10);
    if s.usr.voronoi_colors.is_empty() {
        for _ in 0..tiling.desired_points {
            let rand_h = s.usr.rng.f32();
            let color = macroquad::color::hsl_to_rgb(rand_h, 1.0, 0.5);
            s.usr.voronoi_colors.push(color);
        }
    }
    for (i, growth) in growths.drain(..).enumerate() {
        let color = s.usr.voronoi_colors[i];
        color_tiles(s, growth, color);
    }

    let tiling = &mut s.usr.voronoi_tiling;
    if tiling.done {
        // remove all the debug view single tiles:
        let mut cb = CommandBuffer::new();
        for (entity, _) in s.world.query_mut::<&BuildingMapTile>() {
            cb.despawn(entity);
        }
        cb.run_on(&mut s.world);

        // replace them with finalized, generated textures
        let final_size = screen_height() * 0.9;
        let tile_size = final_size / s.usr.rand_map.square_size as f32;
        let center = Vec2::new(final_size / 2.0, final_size / 2.0);
        let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
        let delta = screen_center - center;
        for (_i, set) in tiling.growth_sets.drain(..).enumerate() {
            // let color = s.usr.voronoi_colors[i];
            let (transform, _drawable_solid, drawable_outline) = generate_texture_from_tileset(&set, tile_size, delta);
            let entity = s.world.spawn((transform, Layer6, drawable_outline, IsTile, Tint { d: NON_HOVERED_TILE_COLOR }));
            fill_tile_position_map(entity, &mut s.usr.tile_positions, &set, tile_size, delta);
        }

        if !tiling.open_set.is_empty() {
            // this means there were islands that were not reached initially.
            // re-run the voronoi with the open set.
            // scale the number of points down proportionally to what was filled.
            // tiling.next_n(&mut s.usr.rng, 2);
            tiling.continue_with_open_set();
        } else {
            s.clear_color = WATER_COLOR;
        }
    }
}

fn generate_texture_bytes_outline(
    set: &HashSet<(i32, i32)>,
    min_x: i32, max_x: i32,
    min_y: i32, max_y: i32,
) -> Vec<u8> {
    let mut bytes = vec![];
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let color = match set.get(&(x, y)) {
                Some(_) => {
                    // check all neighbors: if at least 1 neighbor is empty,
                    // that means we are on the border. so color this in.
                    let mut is_border = false;
                    for (x, y) in VoronoiTiling::get_surrounding((x, y)) {
                        if !set.contains(&(x, y)) {
                            is_border = true;
                            break;
                        }
                    }
                    if is_border { WHITE } else { BLANK }
                }
                None => {
                    BLANK
                },
            };
            let color_arr: [u8; 4] = color.into();
            bytes.extend(color_arr);
        }
    }
    bytes
}

fn generate_texture_bytes_solid(
    set: &HashSet<(i32, i32)>,
    min_x: i32, max_x: i32,
    min_y: i32, max_y: i32,
) -> Vec<u8> {
    let mut bytes = vec![];
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let color = match set.get(&(x, y)) {
                Some(_) => WHITE,
                None => BLANK,
            };
            let color_arr: [u8; 4] = color.into();
            bytes.extend(color_arr);
        }
    }
    bytes
}

/// given a tile set of indices, calceulate their
/// real world position, and fill a map that maps those indices
/// to the entity
fn fill_tile_position_map(
    entity: Entity,
    map: &mut HashMap<(i32, i32), Entity>,
    set: &HashSet<(i32, i32)>,
    tile_size: f32,
    delta: Vec2,
) {
    for (x, y) in set.iter() {
        let start_pt = Vec2::new(*x as f32, *y as f32);
        let pt = start_pt * tile_size;
        let position = pt + delta;
        map.insert((position.x as i32, position.y as i32), entity);
    }
}

/// returns the transform of where the texture should be positioned
/// and 2 drawable textures: (solid, outline)
fn generate_texture_from_tileset(
    set: &HashSet<(i32, i32)>,
    tile_size: f32,
    delta: Vec2,
) -> (Transform, Drawable, Drawable) {
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    for (x, y) in set.iter() {
        let (x, y) = (*x, *y);
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }
    }
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;
    let original_origin = Vec2::new(min_x as f32, min_y as f32);
    let original_corner = original_origin + Vec2::new(width as f32, height as f32);
    let original_dist = original_corner.distance(original_origin);
    let scaled_origin = original_origin * tile_size;
    let scaled_corner = original_corner * tile_size;
    let scaled_dist = scaled_corner.distance(scaled_origin);
    // macroquad::logging::warn!("Scaled dist {}, original dist {}. tile size {}", scaled_dist, original_dist, tile_size);
    let scale = scaled_dist / original_dist;
    let outline_bytes = generate_texture_bytes_outline(&set, min_x, max_x, min_y, max_y);
    let solid_bytes = generate_texture_bytes_solid(&set, min_x, max_x, min_y, max_y);
    let start_pt = Vec2::new(min_x as f32, min_y as f32);
    let pt = start_pt * tile_size;
    let width = width as u16;
    let height = height as u16;
    let new_t_outline = Texture2D::from_rgba8(width, height, &outline_bytes);
    new_t_outline.set_filter(FilterMode::Nearest);
    let new_t_solid = Texture2D::from_rgba8(width, height, &solid_bytes);
    new_t_solid.set_filter(FilterMode::Nearest);
    let position = pt + delta;
    let transform = Transform::from_scale_angle_position(scale, 0.0, position);
    let draw_solid = Drawable::Texture { d: new_t_solid, dont_center: true };
    let draw_outline = Drawable::Texture { d: new_t_outline, dont_center: true };
    (transform, draw_solid, draw_outline)
}

fn color_tiles(
    s: &mut GameState,
    tiles: Vec<(i32, i32)>,
    color: Color
) {
    let d = s.textures[&Textures::empty];
    let d_size = d.width();
    let final_size = screen_height() * 0.9;
    let tile_size = final_size / s.usr.rand_map.square_size as f32;
    let scale = tile_size / d_size;
    let center = Vec2::new(final_size / 2.0, final_size / 2.0);
    let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
    let delta = screen_center - center;

    for (x, y) in tiles {
        let x = x as f32;
        let y = y as f32;
        let x = x * tile_size;
        let y = y * tile_size;
        let pt = Vec2::new(x, y);
        let transform = Transform::from_scale_angle_position(scale, 0.0, pt + delta);
        let tint = Tint { d: color };
        s.world.spawn((transform, Layer9, Drawable::Texture { d, dont_center: false }, tint, BuildingMapTile));
    }
}

fn fill_generated_map(s: &mut GameState, _dt: f32) {
    let next = s.usr.rand_map.get_next();
    let final_size = screen_height() * 0.9;
    let half_size = s.usr.rand_map.square_size as f32 / 2.0;
    let center = Vec2::new(final_size / 2.0, final_size / 2.0);
    let original_center = Vec2::new(half_size, half_size);
    let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
    let tile_size = final_size / s.usr.rand_map.square_size as f32;
    let d = s.textures[&Textures::empty];
    let d_size = d.width();
    let scale = tile_size / d_size;
    let delta = screen_center - center;
    let water_level = 0.20;
    if next.is_empty() && !s.usr.filled.data.is_empty() {
        // make rivers. set usr filled data to be water colored for every
        // tile a river lands on.
        create_rivers(s);
        let mut data = std::mem::take(&mut s.usr.filled.data);
        let size = s.usr.rand_map.square_size as u16;
        let mut bytes = vec![];
        for row in data.drain(..) {
            for val in row {
                let ext: [u8; 4] = val.into();
                bytes.extend(ext);
            }
        }
        let new_t = Texture2D::from_rgba8(size, size, &bytes);
        new_t.set_filter(FilterMode::Nearest);
        let scale = final_size / new_t.width();
        let transform = Transform::from_scale_angle_position(scale, 0.0, screen_center);
        s.world.spawn((transform, Layer4, Drawable::Texture { d: new_t, dont_center: false }));

        let mut cb = CommandBuffer::new();
        for (entity, _) in s.world.query_mut::<&BuildingMapTile>() {
            cb.despawn(entity);
        }
        s.usr.voronoi_tiling.ready_to_tile = true;
        // s.usr.recursive_tiling.ready_to_tile = true;
        cb.run_on(&mut s.world);
    }
    for (x, y, height) in next {
        let y_index = y as usize;
        if s.usr.filled.data.len() == y_index {
            s.usr.filled.data.push(vec![]);
        }
        let row = &mut s.usr.filled.data[y_index];
        let height = height.clamp(-0.5, 0.5);
        let height = height + 0.5;
        let original_xy = (x, y);
        let x = x as f32;
        let y = y as f32;
        let pt: Vec2 = (x, y).into();
        let dist = pt.distance(original_center);
        let water_mult = 1.0 - (1.4 * dist / s.usr.rand_map.square_size as f32).clamp(0.0, 1.0);
        let height = height * water_mult;
        let x = x * tile_size;
        let y = y * tile_size;
        let pt = Vec2::new(x, y);
        let transform = Transform::from_scale_angle_position(scale, 0.0, pt + delta);
        let color = if height < water_level {
            WATER_COLOR
        } else {
            s.usr.voronoi_tiling.open_set.insert(original_xy);
            s.usr.voronoi_tiling.open_set_list.push(original_xy);
            // s.usr.voronoi_tiling.open_set_list.sort();
            // s.usr.recursive_tiling.open_set.insert(original_xy);
            GREEN
        };
        row.push(color);
        let tint = Tint { d: color };
        s.world.spawn((transform, Layer0, tint, Drawable::Texture { d, dont_center: false }, BuildingMapTile));
    }
}

#[macroquad::main("battlegame")]
async fn main() {
    let systems = get_all_systems();
    let state: State<MyState, Textures> = State::new();

    run(state, systems, 100).await;
}
