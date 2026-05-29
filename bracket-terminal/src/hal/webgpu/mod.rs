//! Provides wgpu support back-end.

#[path = "../dummy/keycodes.rs"]
mod keycodes;
pub use keycodes::VirtualKeyCode;

mod platform;
pub use platform::*;
mod init;
pub use init::*;
mod font;
pub use font::*;
mod shader;
pub use shader::*;
mod backend;
pub use backend::*;
mod mainloop;
pub use mainloop::*;
mod backing;
pub(crate) use backing::*;
mod framebuffer;
pub(crate) use framebuffer::*;
mod quadrender;

pub fn log(s: &str) {
    println!("{}", s);
}
