pub mod thread;

use crate::{algorithms::triangle::Triangle, shaders::FragmentShader};

#[derive(Clone, Copy)]
pub struct RenderCommand {
    pub triangle: Triangle,
    pub shader: FragmentShader,
    pub emissive: bool,
}
