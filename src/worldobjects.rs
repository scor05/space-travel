use crate::{model::Model, shaders::FragmentShader};
use raylib::prelude::Vector3;

pub struct World {
    pub meshes: Vec<Model>,
    pub objects: Vec<WorldObject>,
}

pub struct WorldObject {
    pub index: usize,
    pub position: Vector3,
    pub offset: Vector3,
    pub rotation: Vector3,
    pub scale: f32,
    pub shader: FragmentShader,
    pub emissive: bool,
    pub orbit_speed: f32,
    pub rotation_speed: f32,
}
