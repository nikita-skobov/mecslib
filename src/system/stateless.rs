use crate::data::{world::State, loading::TextureEnum};


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

pub fn do_stuff<U: Default, T: TextureEnum>(s: &mut State<U, T>, _dt: f32) {

}

pub fn do_stuff2<U: Default, T: TextureEnum>(s: &mut State<U, T>, _dt: f32) {

}
