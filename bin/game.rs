use hecs::*;
use macroquad::prelude::*;
use mquad_ecs_lib::{
    components::*,
    system::{stateless::*, stateful::*},
    data::{
        loading::TextureEnum,
        world::{State, UserState, run},
    },
    create_texture_enum,
    sys,
};


pub struct MyState {
    pub character: Entity,
    pub rand_map: RandomMapGen,
    pub rng: fastrand::Rng,
}

impl Default for MyState {
    fn default() -> Self {
        Self {
            rng: fastrand::Rng::with_seed(412),
            character: Entity::DANGLING,
            rand_map: Default::default(),
        }
    }
}

impl UserState<Textures> for MyState {
    fn initialize(s: &mut State<Self, Textures>) {
        s.usr.rand_map = RandomMapGen::new(500, 3000, s.usr.rng.u64(0..u64::MAX));

        let transform = Transform::from_scale_angle_position(1.0, 0.0, (0.0, 0.0));
        let draw = Drawable::texture(s, Textures::test);
        let unit = s.world.spawn((transform, draw, Layer1));

        let transform = Transform::from_scale_angle_position(1.0, std::f32::consts::FRAC_PI_4, (10.0, 0.0));
        let draw = Drawable::texture(s, Textures::other);
        let parent = Parent {
            parent: unit,
            local_transform: transform,
        };
        let _other = s.world.spawn((Transform::default(), draw, parent, Layer2));
        s.usr.character = unit;
    }
}

pub type MySystem = System<MyState, Textures>;
pub type GameState = State<MyState, Textures>;

create_texture_enum!(Textures; other, test, empty);


fn get_all_systems() -> &'static [MySystem] {
    &[
        sys!(handle_pan),
        sys!(control_character),
        sys!(update_children_transforms),
        sys!(fill_generated_map),
        sys!(draw),
    ]
}

fn fill_generated_map(s: &mut GameState, _dt: f32) {
    let next = s.usr.rand_map.get_next();
    let final_size = screen_height() * 0.9;
    let center = Vec2::new(final_size / 2.0, final_size / 2.0);
    let screen_center = Vec2::new(screen_width() / 2.0, screen_height() / 2.0);
    let tile_size = final_size / s.usr.rand_map.square_size as f32;
    let d = s.textures[&Textures::empty];
    let d_size = d.width();
    let scale = tile_size / d_size;
    let delta = screen_center - center;
    let water_level = 0.45;
    for (x, y, height) in next {
        let height = height.clamp(-0.5, 0.5);
        let height = height + 0.5;
        let x = x as f32;
        let y = y as f32;
        let x = x * tile_size;
        let y = y * tile_size;
        let pt = Vec2::new(x, y);
        let transform = Transform::from_scale_angle_position(scale, 0.0, pt + delta);
        let color = if height < water_level {
            BLUE
        } else {
            GREEN
        };
        let tint = Tint { d: color };
        s.world.spawn((transform, Layer0, tint, Drawable::Texture { d }));
    }
}

fn control_character(s: &mut GameState, _dt: f32) {
    let d_rotation = 0.01;
    let d_move = 2.0;
    let d_scale = 0.03;

    let mut base_angle = 0.0;
    let mut base_pos = Vec2::ZERO;
    let mut base_scale = 1.0;
    if is_key_down(KeyCode::Q) {
        base_angle -= d_rotation;
    }
    if is_key_down(KeyCode::E) {
        base_angle += d_rotation;
    }
    if is_key_down(KeyCode::W) {
        base_pos.y -= d_move;
    }
    if is_key_down(KeyCode::S) {
        base_pos.y += d_move;
    }
    if is_key_down(KeyCode::A) {
        base_pos.x -= d_move;
    }
    if is_key_down(KeyCode::D) {
        base_pos.x += d_move;
    }
    if is_key_down(KeyCode::Equal) {
        base_scale += d_scale;
    }
    if is_key_down(KeyCode::Minus) {
        base_scale -= d_scale;
    }
    if base_angle == 0.0 && base_pos == Vec2::ZERO && base_scale == 0.0 {
        return;
    }

    let transform = match s.world.query_one_mut::<&mut Transform>(s.usr.character) {
        Ok(x) => x,
        Err(_) => return,
    };
    let movement = Transform::from_scale_angle_position(base_scale, base_angle, base_pos);
    transform.d.matrix2 = transform.d.matrix2 * movement.d.matrix2;
    transform.d.translation += base_pos;
}

#[macroquad::main("battlegame")]
async fn main() {
    let systems = get_all_systems();
    let state: State<MyState, Textures> = State::new();

    run(state, systems, 100).await;
}
