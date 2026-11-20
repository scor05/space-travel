use super::RenderCommand;
use crate::algorithms::scanline::FilledScanline;
use crate::algorithms::triangle::FragmentSample;
use crate::shaders::Uniforms;
use raylib::prelude::Color;
use std::thread as std_thread;

const MAX_RENDER_WORKERS: usize = 8;

pub struct BandBuffer {
    pub start_y: i32,
    pub end_y: i32,
    pub width: i32,
    pub color: Vec<Color>,
    pub emissive: Vec<Color>,
    pub depth: Vec<f32>,
}

impl BandBuffer {
    fn new(width: i32, start_y: i32, end_y: i32, background: Color) -> Self {
        let height = end_y - start_y;
        let pixel_count = width as usize * height as usize;

        Self {
            start_y,
            end_y,
            width,
            color: vec![background; pixel_count],
            emissive: vec![Color::BLANK; pixel_count],
            depth: vec![f32::INFINITY; pixel_count],
        }
    }

    pub fn index(&self, x: i32, y: i32) -> usize {
        debug_assert!(x >= 0 && x < self.width);
        debug_assert!(y >= self.start_y && y < self.end_y);
        (y - self.start_y) as usize * self.width as usize + x as usize
    }

    fn shade_sample(
        &mut self,
        command: &RenderCommand,
        sample: FragmentSample,
        uniforms: &Uniforms,
    ) {
        let index = self.index(sample.x, sample.y);
        if sample.depth >= self.depth[index] {
            return;
        }

        let fragment = command.triangle.interpolate_fragment(sample);
        let color = (command.shader)(&fragment, uniforms);

        self.depth[index] = sample.depth;
        self.color[index] = color;
        self.emissive[index] = if command.emissive {
            color
        } else {
            Color::BLANK
        };
    }
}

fn render_band(
    commands: &[RenderCommand],
    uniforms: &Uniforms,
    width: i32,
    start_y: i32,
    end_y: i32,
    background: Color,
) -> BandBuffer {
    let rasterizer = FilledScanline;
    let mut band = BandBuffer::new(width, start_y, end_y, background);

    for command in commands {
        rasterizer.rasterize(&command.triangle, width, start_y, end_y, |sample| {
            band.shade_sample(command, sample, uniforms)
        });
    }

    band
}

pub fn render_parallel_bands(
    commands: &[RenderCommand],
    uniforms: &Uniforms,
    width: i32,
    height: i32,
    background: Color,
) -> Vec<BandBuffer> {
    assert!(
        width > 0 && height > 0,
        "framebuffer dimensions must be positive"
    );

    let available_workers = std_thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1);
    let worker_count = available_workers
        .min(MAX_RENDER_WORKERS)
        .min(height as usize)
        .max(1);
    let rows_per_band = (height as usize).div_ceil(worker_count);

    std_thread::scope(|scope| {
        let mut handles = Vec::with_capacity(worker_count);

        for worker_index in 0..worker_count {
            let start_y = worker_index * rows_per_band;
            let end_y = ((worker_index + 1) * rows_per_band).min(height as usize);

            if start_y >= end_y {
                continue;
            }

            handles.push(scope.spawn(move || {
                render_band(
                    commands,
                    uniforms,
                    width,
                    start_y as i32,
                    end_y as i32,
                    background,
                )
            }));
        }

        handles
            .into_iter()
            .map(|handle| handle.join().expect("render worker panicked"))
            .collect()
    })
}
