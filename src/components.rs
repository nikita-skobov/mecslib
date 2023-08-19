//! All components are data types that can be found within a hecs world.
//! Most components are simply wrappers over other types where the inner type
//! is accessable by `.d` where d stands for data.

use hecs::*;
use macroquad::prelude::*;

use crate::data::{world::{State, UserState}, loading::TextureEnum};

pub struct Transform {
    pub d: Affine2,
}
impl Transform {
    /// position == translation
    pub fn from_scale_angle_position(scale: f32, angle: f32, position: impl Into<Vec2>) -> Self {
        let d = Affine2::from_scale_angle_translation(
            Vec2::new(scale, scale),
            angle,
            position.into(),
        );
        Self { d }
    }
}

pub struct Parent {
    /// the parent of this entity
    pub parent: Entity,
    /// the local offset this entity has in relation to the parent's offset
    pub local_transform: Transform,
}

/// unit struct indicating an entity is dirty, which means
/// it's child transforms need to be re-calculated
pub struct Dirty;

/// represents anything drawable. currently just limitied to single textures
/// but can be expanded to include shapes, animations, text, etc.
pub enum Drawable {
    Texture { d: Texture2D },
}
impl Drawable {
    pub fn texture<U: UserState<T>, T: TextureEnum>(s: &State<U, T>, t: T) -> Self {
        let d = s.textures[&t];
        Self::Texture { d }
    }
}


/// unit structs representing which layer objects should be drawn on
pub struct Layer0;
pub struct Layer1;
pub struct Layer2;
pub struct Layer3;
pub struct Layer4;
pub struct Layer5;
pub struct Layer6;
pub struct Layer7;
pub struct Layer8;
pub struct Layer9;