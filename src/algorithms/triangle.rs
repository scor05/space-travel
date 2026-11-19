use crate::framebuffer::Framebuffer;
use crate::shaders::{FragmentShader, Uniforms};
use raylib::prelude::{Color, Vector3};

pub const COVERAGE_EPSILON: f32 = 1.0e-5;

#[derive(Clone, Copy)]
pub enum TriangleMode {
    Wireframe,
    FilledBarycentric,
    FilledScanline,
}

#[derive(Clone, Copy)]
pub struct Triangle {
    pub vertices: [Vector3; 3],
    pub local_positions: [Vector3; 3],
    pub world_positions: [Vector3; 3],
    pub world_normals: [Vector3; 3],
    pub colors: [Color; 3],
}

impl Triangle {
    pub fn new(first: Vector3, second: Vector3, third: Vector3) -> Self {
        Self {
            vertices: [first, second, third],
            local_positions: [first, second, third],
            world_positions: [first, second, third],
            world_normals: [Vector3::zero(); 3],
            colors: [Color::RED, Color::GREEN, Color::BLUE],
        }
    }

    pub fn inverse_area(&self) -> Option<f32> {
        let [first, second, third] = self.vertices;
        let area = edge_function(first, second, third.x, third.y);

        if area.abs() <= f32::EPSILON {
            None
        } else {
            Some(1.0 / area)
        }
    }

    pub fn sample_at(&self, x: i32, y: i32, inverse_area: f32) -> Option<FragmentSample> {
        let [first, second, third] = self.vertices;
        let sample_x = x as f32 + 0.5;
        let sample_y = y as f32 + 0.5;

        let weights = [
            edge_function(second, third, sample_x, sample_y) * inverse_area,
            edge_function(third, first, sample_x, sample_y) * inverse_area,
            edge_function(first, second, sample_x, sample_y) * inverse_area,
        ];

        if weights.iter().any(|weight| *weight < -COVERAGE_EPSILON) {
            return None;
        }

        let depth = weights[0] * first.z + weights[1] * second.z + weights[2] * third.z;

        Some(FragmentSample {
            x,
            y,
            depth,
            weights,
        })
    }

    pub fn interpolate_fragment(&self, sample: FragmentSample) -> Fragment {
        let weights = sample.weights;

        Fragment {
            x: sample.x,
            y: sample.y,
            depth: sample.depth,
            local_position: interpolate_vector(self.local_positions, weights),
            world_position: interpolate_vector(self.world_positions, weights),
            world_normal: interpolate_vector(self.world_normals, weights),
            color: interpolate_color(self.colors, weights),
        }
    }

    pub fn fragment_at(&self, x: i32, y: i32) -> Option<Fragment> {
        let [first, second, third] = self.vertices;
        let sample_x = x as f32 + 0.5;
        let sample_y = y as f32 + 0.5;
        let area = edge_function(first, second, third.x, third.y);

        if area.abs() <= f32::EPSILON {
            return None;
        }

        let weights = [
            edge_function(second, third, sample_x, sample_y) / area,
            edge_function(third, first, sample_x, sample_y) / area,
            edge_function(first, second, sample_x, sample_y) / area,
        ];

        if weights.iter().any(|weight| *weight < -COVERAGE_EPSILON) {
            return None;
        }

        let depth = weights[0] * first.z + weights[1] * second.z + weights[2] * third.z;
        let local_position = interpolate_vector(self.local_positions, weights);
        let world_position = interpolate_vector(self.world_positions, weights);
        let world_normal = interpolate_vector(self.world_normals, weights);
        Some(Fragment {
            x,
            y,
            depth,
            local_position,
            world_position,
            world_normal,
            color: interpolate_color(self.colors, weights),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fragment {
    pub x: i32,
    pub y: i32,
    pub depth: f32,

    pub local_position: Vector3,
    pub world_position: Vector3,
    pub world_normal: Vector3,

    pub color: Color,
}

#[derive(Clone, Copy)]
pub struct FragmentSample {
    pub x: i32,
    pub y: i32,
    pub depth: f32,
    pub weights: [f32; 3],
}

pub trait TrianglePipeline {
    fn render(
        &self,
        framebuffer: &mut Framebuffer,
        triangle: &Triangle,
        shader: FragmentShader,
        emissive: bool,
        uniforms: &Uniforms,
    );
}

fn edge_function(start: Vector3, end: Vector3, x: f32, y: f32) -> f32 {
    (end.x - start.x) * (y - start.y) - (end.y - start.y) * (x - start.x)
}

fn interpolate_vector(values: [Vector3; 3], weights: [f32; 3]) -> Vector3 {
    values[0] * weights[0] + values[1] * weights[1] + values[2] * weights[2]
}

fn interpolate_color(colors: [Color; 3], weights: [f32; 3]) -> Color {
    let channel = |values: [u8; 3]| {
        (weights[0] * values[0] as f32
            + weights[1] * values[1] as f32
            + weights[2] * values[2] as f32)
            .clamp(0.0, 255.0) as u8
    };

    Color::new(
        channel([colors[0].r, colors[1].r, colors[2].r]),
        channel([colors[0].g, colors[1].g, colors[2].g]),
        channel([colors[0].b, colors[1].b, colors[2].b]),
        channel([colors[0].a, colors[1].a, colors[2].a]),
    )
}
