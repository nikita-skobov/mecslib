//! A stateless system is one that runs every frame and queries the world
//! and operates on the data that exists in the current frame. A stateless system
//! is represented as a function that takes a State and a delta time, see: `SystemFn`.

use hecs::*;
use macroquad::prelude::*;

use crate::{
    components::*,
    system::stateful::*,
    data::{
        world::*,
        loading::*
    },
};


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

/// iterates over all children who have parents
/// and update the children's absolute transform
pub fn update_children_transforms<U: UserState<T>, T: TextureEnum>(s: &mut State<U, T>, _dt: f32) {
    let mut parents = s.world.query::<&Parent>();
    let parents = parents.view();

    // View of entities that don't have parents, i.e. roots of the transform hierarchy
    let mut roots = s.world.query::<&Transform>().without::<&Parent>();
    let roots = roots.view();

    for (_entity, (parent, absolute)) in s.world.query::<(&Parent, &mut Transform)>().iter() {
        // Walk the hierarchy from this entity to the final entity that doesnt have any parents
        let mut relative_transform = parent.local_transform;
        let mut ancestor = parent.parent;
        while let Some(next) = parents.get(ancestor) {
            relative_transform.d = next.local_transform.d * relative_transform.d;
            ancestor = next.parent;
        }
        if let Some(ancestor_transform) = roots.get(ancestor) {
            absolute.d = ancestor_transform.d * relative_transform.d;
        } // TODO: else log error? how could this happen
    }
}

pub fn handle_pan<U: UserState<T>, T: TextureEnum>(s: &mut State<U, T>, _dt: f32) {
    let coords = &mut s.coords;
    // always allow panning with WASD
    let mut wasd_panned = false;
    if is_key_down(KeyCode::W) {
        wasd_panned = true;
        coords.pan_y -= coords.wasd_pan_by / coords.scale;
    }
    if is_key_down(KeyCode::A) {
        wasd_panned = true;
        coords.pan_x -= coords.wasd_pan_by / coords.scale;
    }
    if is_key_down(KeyCode::S) {
        wasd_panned = true;
        coords.pan_y += coords.wasd_pan_by / coords.scale;
    }
    if is_key_down(KeyCode::D) {
        wasd_panned = true;
        coords.pan_x += coords.wasd_pan_by / coords.scale;
    }
    let (x, y) = mouse_position();

    // prevent double panning if already panned with wasm
    let can_pan = !wasd_panned;
    // handle pan:
    if can_pan {
        if is_mouse_button_pressed(MouseButton::Right) {
            coords.start_pan_x = x;
            coords.start_pan_y = y;
        }
        if is_mouse_button_down(MouseButton::Right) {
            coords.pan_x -= (x - coords.start_pan_x) / coords.scale;
            coords.pan_y -= (y - coords.start_pan_y) / coords.scale;
            coords.start_pan_x = x;
            coords.start_pan_y = y;
        }
    }

    // handle zoom:
    let (wx_before, wy_before) = coords.to_world(x, y);
    let (_, scrolly) = mouse_wheel();
    if scrolly > 0.0 {
        coords.scale *= 1.0 + CoordTransform::SCALE_BY;
    }
    if scrolly < 0.0 {
        coords.scale *= 1.0 - CoordTransform::SCALE_BY;
    }
    if coords.scale < CoordTransform::MIN_SCALE {
        coords.scale = CoordTransform::MIN_SCALE;
    }
    if coords.scale > CoordTransform::MAX_SCALE {
        coords.scale = CoordTransform::MAX_SCALE;
    }
    let (wx_after, wy_after) = coords.to_world(x, y);
    coords.pan_x += wx_before - wx_after;
    coords.pan_y += wy_before - wy_after;
}

/// draw requires entities with the following components:
/// - transform
/// - drawable
/// - layer (0 - 9)
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
