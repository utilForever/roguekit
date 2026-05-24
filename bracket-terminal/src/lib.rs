#![allow(clippy::multiple_crate_versions)]

#[macro_use]
extern crate lazy_static;
mod bterm;
mod consoles;
mod gamestate;
mod hal;
mod initializer;
mod input;
pub mod rex;

#[cfg(test)]
mod test_utils;

pub use bracket_embedding::prelude::{EMBED, embedded_resource, link_resource};

pub type BResult<T> = anyhow::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub(crate) use input::clear_input_state;
pub type FontCharType = u16;
pub use consoles::console;

#[cfg(all(
    any(feature = "opengl", feature = "webgpu"),
    any(feature = "crossterm", feature = "curses")
))]
compile_error!("Default features (opengl) must be disabled for other back-ends");

pub mod prelude {

    pub use crate::BResult;
    pub use crate::FontCharType;
    pub use crate::bterm::*;
    pub use crate::consoles::*;
    pub use crate::gamestate::GameState;
    pub use crate::hal::{BACKEND, BTermPlatform, Font, InitHints, Shader, init_raw};
    pub use crate::initializer::*;
    pub use crate::input::{BEvent, INPUT, Input};
    pub use crate::rex;
    pub use crate::rex::*;
    pub use bracket_color::prelude::*;
    pub use bracket_embedding::prelude::{EMBED, embedded_resource, link_resource};
    pub use bracket_geometry::prelude::*;
    pub use winit::keyboard::KeyCode;
    pub type BError = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    #[cfg(all(feature = "opengl", not(target_arch = "wasm32")))]
    pub use crate::hal::GlCallback;

    #[cfg(any(
        all(feature = "opengl", not(target_arch = "wasm32")),
        all(feature = "webgpu", not(feature = "opengl")),
        target_arch = "wasm32",
        feature = "curses",
        feature = "crossterm",
    ))]
    pub use crate::hal::VirtualKeyCode;
}

#[macro_export]
macro_rules! add_wasm_support {
    () => {
        #[cfg(target_arch = "wasm32")]
        use wasm_bindgen::prelude::*;

        #[cfg(target_arch = "wasm32")]
        #[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
        pub fn wasm_main() {
            main().expect("Error in main");
        }
    };
}
