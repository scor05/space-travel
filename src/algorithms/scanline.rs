use super::triangle::{COVERAGE_EPSILON, FragmentSample, Triangle, TrianglePipeline};
use crate::framebuffer::Framebuffer;
use crate::shaders::{FragmentShader, Uniforms};
use raylib::prelude::Vector3;

pub struct FilledScanline;

impl FilledScanline {
    pub fn rasterize<F>(
        &self,
        triangle: &Triangle,
        width: i32,
        band_min_y: i32,
        band_max_y: i32,
        mut emit_sample: F,
    ) where
        F: FnMut(FragmentSample),
    {
        let [first, second, third] = triangle.vertices;

        let Some(inverse_area) = triangle.inverse_area() else {
            return;
        };

        let calc_min_y = first.y.min(second.y).min(third.y).floor() as i32;
        let calc_max_y = first.y.max(second.y).max(third.y).ceil() as i32;

        let min_y = calc_min_y.clamp(band_min_y, band_max_y);
        let max_y = calc_max_y.clamp(band_min_y, band_max_y);

        if min_y >= max_y {
            return;
        }

        for y in min_y..max_y {
            let sample_y = y as f32 + 0.5;
            let mut left = f32::INFINITY;
            let mut right = f32::NEG_INFINITY;

            include_edge_intersection(&mut left, &mut right, first, second, sample_y);
            include_edge_intersection(&mut left, &mut right, second, third, sample_y);
            include_edge_intersection(&mut left, &mut right, third, first, sample_y);

            if !left.is_finite() || !right.is_finite() {
                continue;
            }

            let first_x = (left - 0.5 - COVERAGE_EPSILON).ceil() as i32;
            let last_x = (right - 0.5 + COVERAGE_EPSILON).floor() as i32;

            let first_x = first_x.max(0);
            let last_x = last_x.min(width - 1);

            if first_x > last_x {
                continue;
            }

            for x in first_x..=last_x {
                if let Some(sample) = triangle.sample_at(x, y, inverse_area) {
                    emit_sample(sample)
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
            0,
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

fn include_edge_intersection(
    left: &mut f32,
    right: &mut f32,
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

    if sample_y < minimum_y || sample_y >= maximum_y {
        return;
    }

    let interpolation = (sample_y - start.y) / delta_y;
    let x = start.x + interpolation * (end.x - start.x);
    *left = left.min(x);
    *right = right.max(x);
}
