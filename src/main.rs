mod algorithms;
mod camera;
mod framebuffer;
mod line;
mod model;
mod parallelism;
mod procedural;
mod shaders;
mod worldobjects;

use algorithms::barycentric::FilledBarycentric;
use algorithms::scanline::FilledScanline;
use algorithms::triangle::{Triangle, TriangleMode, TrianglePipeline};
use algorithms::wireframe::WireFrame;
use camera::Camera;
use framebuffer::Framebuffer;
use model::Model;
use parallelism::*;
use raylib::prelude::*;
use shaders::Uniforms;
use std::f32::consts::PI;
use std::time::Instant;
use worldobjects::WorldObject;

const SHIP_MODEL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/obj/nave.obj");
const SPHERE_MODEL_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/obj/sphere.obj");
const STAR_SCALE: f32 = 50.0;
const STAR_ROTATION_SPEED: f32 = PI / 40.0;
const PLANET_COUNT: usize = 2;

const PLANET_1_SCALE: f32 = 7.0;
const PLANET_2_SCALE: f32 = 12.0;
const PLANET_1_ORBIT_RADIUS: f32 = 4.0;
const PLANET_2_ORBIT_RADIUS: f32 = 30.0;
const PLANET_1_ORBIT_SPEED: f32 = 3.0;
const PLANET_2_ORBIT_SPEED: f32 = 5.0;
const PLANET_1_ROTATION_SPEED: f32 = PI / 20.0;
const PLANET_2_ROTATION_SPEED: f32 = PI / 30.0;

fn matrix_mult(mat: &[Vector3; 3], vec: Vector3) -> Vector3 {
    let mut result = Vector3::zero();
    result.x += mat[0].x * vec.x + mat[0].y * vec.y + mat[0].z * vec.z;
    result.y += mat[1].x * vec.x + mat[1].y * vec.y + mat[1].z * vec.z;
    result.z += mat[2].x * vec.x + mat[2].y * vec.y + mat[2].z * vec.z;

    result
}

fn normalize(vector: Vector3) -> Vector3 {
    let length_squared = vector.length_sqr();
    if length_squared <= f32::EPSILON {
        Vector3::zero()
    } else {
        vector * length_squared.sqrt().recip()
    }
}

fn transform(
    vertex: Vector3,
    translation: Vector3,
    rotx: &[Vector3; 3],
    roty: &[Vector3; 3],
    rotz: &[Vector3; 3],
    scale: f32,
    center: Vector3,
) -> Vector3 {
    let mut new_vertex = vertex;
    new_vertex -= center;

    new_vertex = matrix_mult(rotx, new_vertex);
    new_vertex = matrix_mult(roty, new_vertex);
    new_vertex = matrix_mult(rotz, new_vertex);

    new_vertex *= scale;

    new_vertex += translation;
    new_vertex
}

fn render(
    framebuffer: &mut Framebuffer,
    mesh: &Model,
    object: &WorldObject,
    camera: &Camera,
    pipeline: &dyn TrianglePipeline,
    uniforms: &Uniforms,
) {
    let (sinx, cosx) = object.rotation.x.sin_cos();
    let (siny, cosy) = object.rotation.y.sin_cos();
    let (sinz, cosz) = object.rotation.z.sin_cos();

    let rot_x: [Vector3; 3] = [
        Vector3::new(1.0, 0.0, 0.0),
        Vector3::new(0.0, cosx, -sinx),
        Vector3::new(0.0, sinx, cosx),
    ];
    let rot_y: [Vector3; 3] = [
        Vector3::new(cosy, 0.0, siny),
        Vector3::new(0.0, 1.0, 0.0),
        Vector3::new(-siny, 0.0, cosy),
    ];
    let rot_z: [Vector3; 3] = [
        Vector3::new(cosz, -sinz, 0.0),
        Vector3::new(sinz, cosz, 0.0),
        Vector3::new(0.0, 0.0, 1.0),
    ];

    let transformed_vertices: Vec<(Option<Vector3>, Vector3, Vector3)> = mesh
        .vertices
        .iter()
        .map(|vertex| {
            let world_position = transform(
                *vertex,
                object.position,
                &rot_x,
                &rot_y,
                &rot_z,
                object.scale,
                mesh.center,
            );

            let mut world_normal = normalize(*vertex - mesh.center);
            world_normal = matrix_mult(&rot_x, world_normal);
            world_normal = matrix_mult(&rot_y, world_normal);
            world_normal = normalize(matrix_mult(&rot_z, world_normal));

            let view_position = camera.world_to_view(world_position);
            let screen_position =
                camera.project(view_position, framebuffer.width, framebuffer.height);

            (screen_position, world_position, world_normal)
        })
        .collect();

    for indices in &mesh.triangles {
        let (Some(first), Some(second), Some(third)) = (
            transformed_vertices[indices[0]].0,
            transformed_vertices[indices[1]].0,
            transformed_vertices[indices[2]].0,
        ) else {
            continue;
        };

        let projected = [first, second, third];

        pipeline.render(
            framebuffer,
            &Triangle {
                vertices: projected,
                local_positions: [
                    mesh.vertices[indices[0]],
                    mesh.vertices[indices[1]],
                    mesh.vertices[indices[2]],
                ],
                world_positions: [
                    transformed_vertices[indices[0]].1,
                    transformed_vertices[indices[1]].1,
                    transformed_vertices[indices[2]].1,
                ],
                world_normals: [
                    transformed_vertices[indices[0]].2,
                    transformed_vertices[indices[1]].2,
                    transformed_vertices[indices[2]].2,
                ],
                colors: [
                    mesh.colors[indices[0]],
                    mesh.colors[indices[1]],
                    mesh.colors[indices[2]],
                ],
            },
            object.shader,
            object.emissive,
            uniforms,
        );
    }
}

fn main() {
    let ship_model = Model::load(SHIP_MODEL_PATH)
        .unwrap_or_else(|error| panic!("Could not load {SHIP_MODEL_PATH}: {error}"));
    let sphere_model = Model::load(SPHERE_MODEL_PATH)
        .unwrap_or_else(|error| panic!("Could not load {SPHERE_MODEL_PATH}: {error}"));

    let window_width = 600;
    let window_height = 400;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Space Travel :D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();
    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);

    let mut display_texture = window
        .load_texture_from_image(&raylib_thread, &framebuffer.color_buffer)
        .expect("Error loading texture from framebuffer");

    let wireframe = WireFrame;
    let scanline = FilledScanline;
    let barycentric = FilledBarycentric;

    let mode = TriangleMode::FilledScanline;
    let pipeline: &dyn TrianglePipeline = match mode {
        TriangleMode::Wireframe => &wireframe,
        TriangleMode::FilledBarycentric => &barycentric,
        TriangleMode::FilledScanline => &scanline,
    };

    let shaders = shaders::registry();
    let start_time = Instant::now();
    let system_center: Vector3 = Vector3::new(10.0, 0.0, 10.0);
    let mut camera = Camera::new(Vector3::new(10.0, 0.0, -140.0));

    // plano elíptico como y=0
    // TODO: sumarle 2 para contar la nave también
    let meshes: [&Model; PLANET_COUNT + 1] = [&sphere_model, &sphere_model, &sphere_model];

    // la estrella es el primer elemento, con base a él se calculan todos
    let objects: [&WorldObject; PLANET_COUNT + 1] = [
        &WorldObject {
            index: 0,
            position: system_center,
            scale: STAR_SCALE,
            rotation: Vector3::zero(),
            shader: shaders["Neutron star"],
            emissive: true,
            orbit_radius: 0.0,
            orbit_speed: 0.0,
            rotation_speed: STAR_ROTATION_SPEED,
        },
        &WorldObject {
            index: 1,
            position: system_center
                + Vector3::new(
                    -(STAR_SCALE + PLANET_1_ORBIT_RADIUS),
                    0.0,
                    -(STAR_SCALE + PLANET_1_ORBIT_RADIUS),
                ),
            rotation: Vector3::zero(),
            scale: PLANET_1_SCALE,
            shader: shaders["Ocean planet"],
            emissive: false,
            orbit_radius: PLANET_1_ORBIT_RADIUS,
            orbit_speed: PLANET_1_ORBIT_SPEED,
            rotation_speed: PLANET_1_ROTATION_SPEED,
        },
        &WorldObject {
            index: 2,
            position: system_center
                + Vector3::new(
                    STAR_SCALE + PLANET_2_ORBIT_RADIUS,
                    0.0,
                    -(STAR_SCALE + PLANET_2_ORBIT_RADIUS),
                ),
            rotation: Vector3::zero(),
            scale: PLANET_2_SCALE,
            shader: shaders["Gas giant"],
            emissive: false,
            orbit_radius: PLANET_2_ORBIT_RADIUS,
            orbit_speed: PLANET_2_ORBIT_SPEED,
            rotation_speed: PLANET_2_ROTATION_SPEED,
        },
    ];

    while !window.window_should_close() {
        camera.update(&window, window.get_frame_time());

        framebuffer.clear();

        let uniforms = Uniforms {
            time: start_time.elapsed().as_secs_f32(),
            light_position: system_center,
            camera_position: camera.position,
        };

        for (mesh, obj) in meshes.iter().zip(objects.iter()) {
            render(&mut framebuffer, mesh, obj, &camera, pipeline, &uniforms);
        }

        framebuffer.apply_bloom();
        framebuffer.swap_buffers(&mut window, &raylib_thread, &mut display_texture);
    }
}
