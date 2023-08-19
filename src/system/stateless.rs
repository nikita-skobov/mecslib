//! A stateless system is one that runs every frame and queries the world
//! and operates on the data that exists in the current frame. A stateless system
//! is represented as a function that takes a State and a delta time, see: `SystemFn`.

use hecs::*;
use macroquad::prelude::*;

use crate::{data::{world::{State, UserState}, loading::TextureEnum}, components::*};


type SystemFn<U, T> = fn(&mut State<U, T>, f32);
type SystemName = &'static str;
/// convenience type alias for your system functions.
/// The output of the `sys` macro outputs a System<U, T> type.
/// It is recommended to define your own type alias as follows:
/// ```
/// pub type MySystem = System<MyUserState, Textures>;
/// ```
/// And then you can use this conveniently like so:
/// ```
/// fn get_all_systems() -> &'static [MySystem] {
///     &[
///         sys!(do_stuff),
///     ]
/// }
/// ```
pub type System<U, T> = (SystemFn<U, T>, SystemName);

/// helper macro to create a system tuple consisting
/// of a function, and the stringified name of that function
#[macro_export]
macro_rules! sys {
    ($f:expr) => {
        ($f, stringify!($f))
    };
}

pub fn do_stuff<U: UserState<T>, T: TextureEnum>(_s: &mut State<U, T>, _dt: f32) {

}

pub fn draw<U: UserState<T>, T: TextureEnum>(s: &mut State<U, T>, dt: f32) {
    draw_layer::<_, _, Layer0>(s, dt);
    draw_layer::<_, _, Layer1>(s, dt);
    draw_layer::<_, _, Layer2>(s, dt);
    draw_layer::<_, _, Layer3>(s, dt);
    draw_layer::<_, _, Layer4>(s, dt);
    draw_layer::<_, _, Layer5>(s, dt);
    draw_layer::<_, _, Layer6>(s, dt);
    draw_layer::<_, _, Layer7>(s, dt);
    draw_layer::<_, _, Layer8>(s, dt);
    draw_layer::<_, _, Layer9>(s, dt);
}

pub fn draw_layer<U: UserState<T>, T: TextureEnum, Layer: Component>(s: &mut State<U, T>, _dt: f32) {
    for (_, (transform, drawable)) in s.world.query_mut::<(&Transform, &Drawable)>().with::<&Layer>() {
        let pt = transform.d.transform_point2(Vec2::ZERO);
        let dir_vec = transform.d.transform_vector2(Vec2::NEG_Y);
        let dir_vec_magnitude = dir_vec.length();
        
        match drawable {
            Drawable::Texture { d } => {
                let width = d.width() * dir_vec_magnitude;
                let height = d.height() * dir_vec_magnitude;
                draw_texture_ex(*d, pt.x - width / 2.0, pt.y - height / 2.0, WHITE, DrawTextureParams {
                    rotation: -dir_vec.angle_between(Vec2::NEG_Y),
                    dest_size: Vec2::new(width, height).into(),
                    ..Default::default()
                });
            }
        }
    }
}
