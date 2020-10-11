//pub mod common;
//use common::bounded_fifo::BoundedFifo;
#[macro_use]
extern crate glium;
mod glium_renderer;
pub mod shaders;

use self::glium_renderer::glium_renderer::GliumRenderer;
#[allow(unused_imports)]
use glium::{glutin, Surface};


fn main() {
    let r = GliumRenderer::new();
    r.run();
}
