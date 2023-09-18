use macroquad::prelude::*;
use std::{collections::HashMap, hash::Hash};

/// provide a name for your enum, and a list of comma separated names of textures.
/// this macro then creates an enum where the variants of
/// the enum are named exactly as passed into the macro,
/// and additionally creates a loader function that will call
/// `include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", stringify!($x), ".png"))`
/// for each texture name variant.
/// 
/// Example:
/// ```
/// create_texture_enum(Hello; a, b)
/// // expects assets/a.png, and assets/b.png to exist at the root of your project.
/// // Hello enum has 2 variants: a and b.
/// ```
#[macro_export]
macro_rules! create_texture_enum {
    ($name:ident; $($x:ident),*) => {
        #[allow(non_camel_case_types)]
        #[derive(PartialEq, Eq, Hash, Clone, Copy)]
        pub enum $name {
            $(
                $x,
            )*
        }

        impl TextureEnum for $name {
            fn load() -> std::collections::HashMap<Self, Texture2D> {
                let mut map: std::collections::HashMap<Self, Texture2D> = Default::default();
                $(
                    let t = Texture2D::from_file_with_format(
                        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/", stringify!($x), ".png")),
                        ImageFormat::Png.into()
                    );
                    // needed to prevent pixelart blur
                    t.set_filter(FilterMode::Nearest);
                    map.insert(Self::$x, t);                    
                )*
                map
            }
        }
    };
}

pub trait TextureEnum: Eq + PartialEq + Hash {
    fn load() -> HashMap<Self, Texture2D>
        where Self: Sized;
}
