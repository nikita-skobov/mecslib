use hecs::*;
use macroquad::prelude::*;
use mquad_ecs_lib::{
    components::*,
    system::stateless::*,
    data::{
        loading::TextureEnum,
        world::{State, UserState, run, generate_triangle_map},
    },
    create_texture_enum,
    sys,
};


pub struct MyState {
    pub character: Entity,
}

impl Default for MyState {
    fn default() -> Self {
        Self {
            character: Entity::DANGLING
        }
    }
}

impl UserState<Textures> for MyState {
    fn initialize(s: &mut State<Self, Textures>) {
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

create_texture_enum!(Textures; other, test);


fn get_all_systems() -> &'static [MySystem] {
    &[
        sys!(handle_pan),
        sys!(control_character),
        sys!(update_children_transforms),
        sys!(draw),
    ]
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
