use super::triangle::{COVERAGE_EPSILON, FragmentSample, Triangle, TrianglePipeline};
use crate::framebuffer::Framebuffer;
use crate::shaders::{FragmentShader, Uniforms};
use raylib::error::InvalidMeshError;
use raylib::prelude::Vector3;

pub struct FilledScanline;

impl FilledScanline {
    pub fn rasterize<F>(&self, triangle: &Triangle, width: i32, height: i32, mut shade_fragment: F)
    where
        F: FnMut(FragmentSample),
    {
        let [first, second, third] = triangle.vertices;

        let inverse_area = triangle.inverse_area().unwrap_or_default();

        if signed_area(first, second, third).abs() <= f32::EPSILON {
            return;
        }

        let calc_min_y = first.y.min(second.y).min(third.y).floor() as i32;
        let calc_max_y = first.y.max(second.y).max(third.y).ceil() as i32;

        let min_y = calc_min_y.clamp(0, height);
        let max_y = calc_max_y.clamp(0, height);

        if min_y >= max_y {
            return;
        }

        for y in min_y..max_y {
            let sample_y = y as f32 + 0.5;
            let mut intersections = Vec::with_capacity(6);

            add_edge_intersections(&mut intersections, first, second, sample_y);
            add_edge_intersections(&mut intersections, second, third, sample_y);
            add_edge_intersections(&mut intersections, third, first, sample_y);

            if intersections.is_empty() {
                continue;
            }

            intersections.sort_by(f32::total_cmp);
            let left = intersections[0];
            let right = intersections[intersections.len() - 1];

            let first_x = (left - 0.5 - COVERAGE_EPSILON).ceil() as i32;
            let last_x = (right - 0.5 + COVERAGE_EPSILON).floor() as i32;

            let first_x = first_x.max(0);
            let last_x = last_x.min(width - 1);

            if first_x > last_x {
                continue;
            }

            for x in first_x..=last_x {
                if let Some(sample) = triangle.sample_at(x, y, inverse_area) {
                    shade_fragment(sample)
                }
            }
        }
    }
}

impl TrianglePipeline for FilledScanline {
    fn render(
        &self,
        framebuffer: &mut Framebuffer,
        triangle: &Triangle,
        shader: FragmentShader,
        emissive: bool,
        uniforms: &Uniforms,
    ) {
        self.rasterize(
            triangle,
            framebuffer.width as i32,
            framebuffer.height as i32,
            |sample| {
                framebuffer.shade_fragment(sample.x, sample.y, sample.depth, emissive, || {
                    let fragment = triangle.interpolate_fragment(sample);
                    shader(&fragment, uniforms)
                });
            },
        );
    }
}

fn add_edge_intersections(
    intersections: &mut Vec<f32>,
    start: Vector3,
    end: Vector3,
    sample_y: f32,
) {
    let delta_y = end.y - start.y;

    if delta_y.abs() <= f32::EPSILON {
        return;
    }

    let minimum_y = start.y.min(end.y);
    let maximum_y = start.y.max(end.y);

    if sample_y < minimum_y - COVERAGE_EPSILON || sample_y > maximum_y + COVERAGE_EPSILON {
        return;
    }

    let interpolation = (sample_y - start.y) / delta_y;
    intersections.push(start.x + interpolation * (end.x - start.x));
}

fn signed_area(first: Vector3, second: Vector3, third: Vector3) -> f32 {
    (second.x - first.x) * (third.y - first.y) - (second.y - first.y) * (third.x - first.x)
}
