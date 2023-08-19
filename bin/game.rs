use macroquad::prelude::*;
use mquad_ecs_lib::{system::stateless::{System, do_stuff}, data::{loading::TextureEnum, world::{State, run, UserState}}, create_texture_enum, sys};

#[derive(Default)]
pub struct MyState {}

impl UserState for MyState {
    fn initialize<T: TextureEnum>(_s: &mut State<Self, T>) {
        
    }
}

pub type MySystem = System<MyState, Textures>;

create_texture_enum!(Textures; other, test);


fn get_all_systems() -> &'static [MySystem] {
    &[
        sys!(do_stuff),
    ]
}

#[macroquad::main("battlegame")]
async fn main() {
    let systems = get_all_systems();
    let state: State<MyState, Textures> = State::new();

    run(state, systems, 0).await;

    // let t = Texture2D::from_file_with_format(include_bytes!("../assets/test.png"), None);
    // let t2 = Texture2D::from_file_with_format(include_bytes!("../assets/other.png"), None);
    // let mut base_angle = std::f32::consts::FRAC_PI_4 * 1.0;
    // let mut base_pos = Vec2::new(300.0, 300.0);
    // let mut base_scale = 4.0;
    // let d_rotation = 0.01;
    // let d_move = 2.0;
    // let d_scale = 0.03;
    // loop {
    //     if is_key_down(KeyCode::Q) {
    //         base_angle -= d_rotation;
    //     }
    //     if is_key_down(KeyCode::E) {
    //         base_angle += d_rotation;
    //     }
    //     if is_key_down(KeyCode::W) {
    //         base_pos.y -= d_move;
    //     }
    //     if is_key_down(KeyCode::S) {
    //         base_pos.y += d_move;
    //     }
    //     if is_key_down(KeyCode::A) {
    //         base_pos.x -= d_move;
    //     }
    //     if is_key_down(KeyCode::D) {
    //         base_pos.x += d_move;
    //     }
    //     if is_key_down(KeyCode::Equal) {
    //         base_scale += d_scale;
    //     }
    //     if is_key_down(KeyCode::Minus) {
    //         base_scale -= d_scale;
    //     }
    //     let transform = Affine2::from_scale_angle_translation(
    //         Vec2::new(base_scale, base_scale),
    //         base_angle,
    //         base_pos,
    //     );
    //     let child = Affine2::from_angle_translation(std::f32::consts::FRAC_PI_4, Vec2::new(20.0, 0.0));
    //     let child_draw = transform * child;
    //     let source_pt = Vec2::NEG_Y;
    //     let pt = transform.transform_point2(source_pt);
    //     let dir_vec = transform.transform_vector2(Vec2::NEG_Y);
    //     let scale = 100.0;
    //     clear_background(BLACK);
    //     draw_texture_affine(t, &transform);
    //     draw_texture_affine(t2, &child_draw);
    //     // draw_line(0.0, 300.0, 300.0, 300.0, 1.0, RED);
    //     // draw_line(300.0, 0.0, 300.0, 300.0, 1.0, RED);
    //     // draw_texture_ex(t, pt.x, pt.y, WHITE, DrawTextureParams {
    //     //     rotation: dir_vec.angle_between(Vec2::NEG_Y), ..Default::default()
    //     // });
    //     draw_circle(pt.x, pt.y, 3.0, RED);
    //     // draw_line(origin.x, origin.y, origin.x + pt.x, origin.y + pt.y, 2.0, WHITE);
    //     draw_line(pt.x, pt.y, pt.x + dir_vec.x * scale, pt.y + dir_vec.y * scale, 2.0, GREEN);
    //     next_frame().await;
    // }
}

// fn draw_texture_affine(t: Texture2D, affine: &Affine2) {
//     let pt = affine.transform_point2(Vec2::NEG_Y);
//     let dir_vec = affine.transform_vector2(Vec2::NEG_Y);
//     let dir_vec_magnitude = dir_vec.length();
//     let width = t.width() * dir_vec_magnitude;
//     let height = t.height() * dir_vec_magnitude;
//     // macroquad::logging::warn!("SIZE OF TEXTURE {}x{}", width, height);
//     draw_texture_ex(t, pt.x - width / 2.0, pt.y - height / 2.0, WHITE, DrawTextureParams {
//         rotation: -dir_vec.angle_between(Vec2::NEG_Y),
//         dest_size: Vec2::new(width, height).into(),
//         ..Default::default()
//     });
// }
