use crate::algorithms::triangle::Fragment;
use crate::procedural::{cellular, fbm, smoothstep};
use raylib::prelude::{Color, Vector3};
use std::collections::HashMap;

pub struct Uniforms {
    pub time: f32,
    pub light_position: Vector3,
    pub camera_position: Vector3,
}

pub type FragmentShader = fn(&Fragment, &Uniforms) -> Color;

pub const STAR_SHADER: &str = "Neutron star";
pub const ROCKY_PLANET_SHADER: &str = "Ocean planet";
pub const GAS_GIANT_SHADER: &str = "Gas giant";
pub const GREEN_PLANET_SHADER: &str = "Green planet";
pub const PURPLE_PLANET_SHADER: &str = "Purple planet";
pub const SHIP_SHADER: &str = "Ship";
pub const RICK_PLANET_SHADER: &str = "Rick planet";

pub fn registry() -> HashMap<String, FragmentShader> {
    HashMap::from([
        (STAR_SHADER.to_owned(), star as FragmentShader),
        (
            ROCKY_PLANET_SHADER.to_owned(),
            rocky_planet as FragmentShader,
        ),
        (GAS_GIANT_SHADER.to_owned(), gas_giant as FragmentShader),
        (
            GREEN_PLANET_SHADER.to_owned(),
            green_planet as FragmentShader,
        ),
        (
            PURPLE_PLANET_SHADER.to_owned(),
            purple_planet as FragmentShader,
        ),
        (SHIP_SHADER.to_owned(), ship as FragmentShader),
        (
            RICK_PLANET_SHADER.to_owned(),
            rick_planet as FragmentShader,
        ),
    ])
}

fn ship(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = fragment.local_position;
    let mut normal = normalize(fragment.world_normal);
    let view_direction = normalize(uniforms.camera_position - fragment.world_position);
    let light_direction = normalize(uniforms.light_position - fragment.world_position);

    if dot(normal, view_direction) < 0.0 {
        normal = -normal;
    }

    let wrapped_diffuse = ((dot(normal, light_direction) + 0.40) / 1.40).clamp(0.0, 1.0);
    let half_direction = normalize(light_direction + view_direction);
    let specular = dot(normal, half_direction).max(0.0).powf(20.0) * wrapped_diffuse;
    let rim = (1.0 - dot(normal, view_direction).max(0.0))
        .powf(2.0)
        .clamp(0.0, 1.0);

    let panel_variation =
        ((position.x * 1.73).sin() * (position.z * 2.11).cos() * 0.5 + 0.5).clamp(0.0, 1.0);
    let mut hull = mix(
        ColorF::rgb(0.12, 0.14, 0.18),
        ColorF::rgb(0.36, 0.39, 0.44),
        0.24 + panel_variation * 0.60,
    );

    let longitudinal_seam = repeating_line(position.x + position.z * 0.18, 2.8, 0.025);
    let cross_seam = repeating_line(position.z - position.x * 0.10, 2.3, 0.025);
    let seams = longitudinal_seam.max(cross_seam);
    hull = mix(hull, ColorF::rgb(0.025, 0.035, 0.055), seams * 0.72);

    let stripe_band = 1.0 - smoothstep(0.12, 0.30, (position.y - 0.15).abs());
    let stripe_segments = smoothstep(0.18, 0.48, (position.x * 1.35).sin().abs());
    let stripe = stripe_band * (0.30 + stripe_segments * 0.70);
    hull = mix(hull, ColorF::rgb(0.025, 0.42, 0.58), stripe * 0.78);

    let scratch_wave = (position.x * 9.7 + position.z * 3.1).sin().abs();
    let scratches = smoothstep(0.975, 0.997, scratch_wave) * (1.0 - stripe_band);
    hull = mix(hull, ColorF::rgb(0.52, 0.55, 0.57), scratches * 0.24);

    let illumination = 0.46 + wrapped_diffuse * 0.48;
    let lit_hull = hull * illumination;

    let pulse = 0.88 + (uniforms.time * 2.4).sin() * 0.12;
    let emissive_stripe = ColorF::rgb(0.02, 0.36, 0.62) * (stripe * pulse * 0.42);
    let highlight = ColorF::rgb(0.82, 0.88, 0.96) * (specular * 0.32);
    let rim_light = ColorF::rgb(0.10, 0.28, 0.46) * (rim * 0.24);

    (lit_hull + emissive_stripe + highlight + rim_light).to_color()
}

fn star(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.local_position);
    let time = uniforms.time;

    let warp = fbm(
        position * 3.2 + Vector3::new(time * 0.15, -time * 0.13, time * 0.10),
        3,
        17,
    );
    let cellular_point =
        position * 8.0 + Vector3::new(warp * 1.7, warp * 0.8 + time * 0.12, -warp * 1.2);
    let cells = cellular(cellular_point, 91);
    let hot_cell_centers = 1.0 - smoothstep(0.12, 0.72, cells);

    let turbulence = fbm(
        position * 5.0 + Vector3::new(-time * 0.08, time * 0.06, time * 0.03),
        3,
        203,
    );

    let base = mix(
        ColorF::rgb(0.008, 0.015, 0.16),
        ColorF::rgb(0.04, 0.38, 1.0),
        0.35 + turbulence * 0.55,
    );
    let granulated = combine(
        ColorF::rgb(0.28, 0.78, 1.0),
        ColorF::BLACK,
        ColorF::splat(hot_cell_centers * 0.62),
        base,
    );
    let flicker = 0.5 + 0.5 * (time * 1.7 + warp * 8.0).sin();
    let active_surface = combine(
        ColorF::rgb(0.72, 0.94, 1.0),
        ColorF::BLACK,
        ColorF::splat(turbulence * (0.08 + flicker * 0.08)),
        granulated,
    );

    active_surface.to_color()
}

fn rocky_planet(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.local_position);
    let normal = normalize(fragment.world_normal);
    let light_direction = normalize(uniforms.light_position - fragment.world_position);

    let continent_warp = fbm(position * 1.8, 3, 300);
    let terrain = fbm(
        position * 3.6 + Vector3::new(continent_warp * 1.3, -continent_warp * 0.7, continent_warp),
        4,
        431,
    );
    let detail = fbm(position * 13.0, 2, 991);
    let land_mask = smoothstep(0.49, 0.59, terrain);

    let ocean = mix(
        ColorF::rgb(0.008, 0.035, 0.13),
        ColorF::rgb(0.015, 0.22, 0.34),
        detail,
    );
    let moisture = fbm(position * 5.5 + Vector3::new(7.0, 2.0, -4.0), 3, 712);
    let vegetation = smoothstep(0.42, 0.68, moisture) * (1.0 - position.y.abs());
    let land = mix(
        ColorF::rgb(0.48, 0.27, 0.08),
        ColorF::rgb(0.035, 0.30, 0.10),
        vegetation,
    );
    let mut surface = mix(ocean, land, land_mask);

    let ice_mask = smoothstep(0.68, 0.84, position.y.abs() + (detail - 0.5) * 0.12);
    surface = mix(surface, ColorF::rgb(0.72, 0.88, 0.94), ice_mask);

    let cloud_point = rotate_y(position, uniforms.time * 0.1);
    let clouds = fbm(
        cloud_point * 7.5 + Vector3::new(uniforms.time * 0.25, 0.0, 0.0),
        3,
        1501,
    );
    let cloud_mask = smoothstep(0.60, 0.72, clouds);

    let shadow_clouds = fbm(
        (cloud_point - light_direction * 0.075) * 7.5
            + Vector3::new(uniforms.time * 0.025, 0.0, 0.0),
        3,
        1501,
    );
    let shadow_mask = smoothstep(0.60, 0.72, shadow_clouds) * 0.28;
    surface = combine(
        surface,
        ColorF::BLACK,
        ColorF::splat(1.0 - shadow_mask),
        ColorF::BLACK,
    );

    let diffuse = dot(normal, light_direction).max(0.0);
    let illumination = 0.10 + diffuse * 0.90;
    surface = combine(
        surface,
        ColorF::BLACK,
        ColorF::splat(illumination),
        ColorF::rgb(0.0, 0.008, 0.018),
    );

    let reflected = reflect(-light_direction, normal);
    let view_direction = normalize(uniforms.camera_position - fragment.world_position);
    let specular = dot(reflected, view_direction).max(0.0).powf(48.0) * (1.0 - land_mask) * diffuse;
    surface = combine(
        ColorF::rgb(0.68, 0.90, 1.0),
        ColorF::BLACK,
        ColorF::splat(specular * 0.8),
        surface,
    );

    let cloud_light = 0.28 + diffuse * 0.72;
    let clouds_color = ColorF::rgb(0.86, 0.91, 0.92) * cloud_light;
    mix(surface, clouds_color, cloud_mask * 0.82).to_color()
}

fn gas_giant(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.local_position);
    let normal = normalize(fragment.world_normal);
    let flowing = rotate_y(position, uniforms.time * 0.15);
    let broad_noise = fbm(
        flowing * 4.0 + Vector3::new(uniforms.time * 0.3, 0.0, 0.0),
        3,
        2401,
    );

    let storm_x = position.x - 0.34;
    let storm_y = position.y + 0.14;
    let storm_distance = (storm_x * storm_x * 1.6 + storm_y * storm_y * 5.0).sqrt();
    let front_facing = 1.0 - smoothstep(-0.72, -0.15, position.z);
    let storm_influence = (1.0 - smoothstep(0.08, 0.48, storm_distance)) * front_facing;
    let storm_angle = storm_y.atan2(storm_x);

    let warped_latitude = position.y
        + (broad_noise - 0.5) * 0.13
        + storm_influence * (storm_angle * 2.0 + uniforms.time * 0.65).sin() * 0.10;
    let band_wave = (warped_latitude * 29.0 + broad_noise * 4.5).sin() * 0.5 + 0.5;
    let soft_band = smoothstep(0.12, 0.88, band_wave);

    let pale = ColorF::rgb(0.84, 0.64, 0.38);
    let rust = ColorF::rgb(0.43, 0.12, 0.055);
    let cream = ColorF::rgb(0.96, 0.82, 0.57);
    let base = mix(rust, pale, soft_band);
    let mut atmosphere = combine(
        cream,
        ColorF::BLACK,
        ColorF::splat((broad_noise - 0.35).max(0.0) * 0.34),
        base,
    );

    let storm_rings =
        (storm_distance * 46.0 - storm_angle * 3.0 + uniforms.time * 0.9).sin() * 0.5 + 0.5;
    let storm_color = mix(
        ColorF::rgb(0.55, 0.10, 0.035),
        ColorF::rgb(1.0, 0.65, 0.28),
        storm_rings,
    );
    atmosphere = mix(atmosphere, storm_color, storm_influence * 0.82);

    let wisps = fbm(flowing * 15.0, 2, 3217);
    atmosphere = combine(
        ColorF::rgb(0.90, 0.70, 0.45),
        ColorF::BLACK,
        ColorF::splat((wisps - 0.5).max(0.0) * 0.18),
        atmosphere,
    );

    let light_direction = normalize(uniforms.light_position - fragment.world_position);
    let diffuse = dot(normal, light_direction).max(0.0);
    combine(
        atmosphere,
        ColorF::BLACK,
        ColorF::splat(0.13 + diffuse * 0.87),
        ColorF::rgb(0.008, 0.003, 0.012),
    )
    .to_color()
}

fn green_planet(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.local_position);
    let normal = normalize(fragment.world_normal);
    let light_direction = normalize(uniforms.light_position - fragment.world_position);

    let vegetation = fbm(position * 5.0, 4, 4103);
    let detail = fbm(position * 22.0, 2, 4211);
    let forest = mix(
        ColorF::rgb(0.025, 0.18, 0.045),
        ColorF::rgb(0.24, 0.72, 0.10),
        smoothstep(0.30, 0.70, vegetation),
    );
    let surface = mix(
        forest,
        ColorF::rgb(0.48, 0.80, 0.20),
        smoothstep(0.55, 0.75, detail) * 0.45,
    );

    let diffuse = dot(normal, light_direction).max(0.0);
    (surface * (0.15 + diffuse * 0.85)).to_color()
}

fn purple_planet(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.local_position);
    let normal = normalize(fragment.world_normal);
    let light_direction = normalize(uniforms.light_position - fragment.world_position);

    let warp = fbm(position * 4.0, 3, 5107);
    let terrain = fbm(
        position * 7.0 + Vector3::new(warp, -warp * 0.7, warp * 0.5),
        3,
        5227,
    );
    // Fixed fissures in the crust, with heat flowing inside them.
    let fissure_distance = (terrain - 0.5).abs();
    let cracks = 1.0 - smoothstep(0.012, 0.045, fissure_distance);
    let hot_core = 1.0 - smoothstep(0.003, 0.018, fissure_distance);

    // Lava lake inside the large front crater of planeta_morado.obj.
    let crater_direction = normalize(Vector3::new(0.2, 0.3, -1.0));
    let crater_distance = dot(position, crater_direction).clamp(-1.0, 1.0).acos();
    let lava_lake = 1.0 - smoothstep(0.16, 0.25, crater_distance);
    let lava_mask = cracks.max(lava_lake);
    let flow = fbm(
        position * 12.0 + Vector3::new(0.0, -uniforms.time * 0.4, uniforms.time * 0.15),
        2,
        5309,
    );
    let pulse = 0.5 + 0.5 * (uniforms.time * 2.0 + warp * 9.0).sin();
    let rock = mix(
        ColorF::rgb(0.035, 0.018, 0.045),
        ColorF::rgb(0.16, 0.08, 0.18),
        warp,
    );
    let diffuse = dot(normal, light_direction).max(0.0);
    let lit_rock = rock * (0.18 + diffuse * 0.82);
    let lava = mix(
        ColorF::rgb(0.85, 0.055, 0.005),
        ColorF::rgb(1.0, 0.85, 0.20),
        hot_core.max(lava_lake) * (0.35 + flow * 0.50 + pulse * 0.15),
    );

    // Lava remains bright on the night side; only the rock receives sunlight.
    mix(lit_rock, lava, lava_mask).to_color()
}

#[derive(Clone, Copy)]
struct ColorF {
    r: f32,
    g: f32,
    b: f32,
}

impl ColorF {
    const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);

    const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    const fn splat(value: f32) -> Self {
        Self::rgb(value, value, value)
    }

    fn to_color(self) -> Color {
        Color::new(
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8,
            255,
        )
    }
}

impl std::ops::Mul<f32> for ColorF {
    type Output = Self;

    fn mul(self, factor: f32) -> Self::Output {
        Self::rgb(self.r * factor, self.g * factor, self.b * factor)
    }
}

impl std::ops::Add for ColorF {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::rgb(self.r + other.r, self.g + other.g, self.b + other.b)
    }
}

fn combine(a: ColorF, b: ColorF, c: ColorF, d: ColorF) -> ColorF {
    ColorF::rgb(
        ((a.r - b.r) * c.r + d.r).clamp(0.0, 1.0),
        ((a.g - b.g) * c.g + d.g).clamp(0.0, 1.0),
        ((a.b - b.b) * c.b + d.b).clamp(0.0, 1.0),
    )
}

fn mix(start: ColorF, end: ColorF, amount: f32) -> ColorF {
    combine(end, start, ColorF::splat(amount), start)
}

fn repeating_line(value: f32, spacing: f32, width: f32) -> f32 {
    let phase = (value / spacing).rem_euclid(1.0);
    let distance_to_line = phase.min(1.0 - phase);
    1.0 - smoothstep(width, width * 2.4, distance_to_line)
}

fn normalize(vector: Vector3) -> Vector3 {
    let length_squared = dot(vector, vector);
    if length_squared <= f32::EPSILON {
        Vector3::zero()
    } else {
        vector * length_squared.sqrt().recip()
    }
}

fn dot(first: Vector3, second: Vector3) -> f32 {
    first.x * second.x + first.y * second.y + first.z * second.z
}

fn reflect(direction: Vector3, normal: Vector3) -> Vector3 {
    direction - normal * (2.0 * dot(direction, normal))
}

fn rotate_y(point: Vector3, angle: f32) -> Vector3 {
    let (sin, cos) = angle.sin_cos();
    Vector3::new(
        point.x * cos + point.z * sin,
        point.y,
        -point.x * sin + point.z * cos,
    )
}

// Planeta naranja con cuencas verdes (planet1.obj). El color sale de la altura
// real de la malla: radio - 0.5 < 0 dentro de las cuencas, > 0 en los bordes.
fn rick_planet(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let position = normalize(fragment.local_position);
    let normal = normalize(fragment.world_normal);
    let light_direction = normalize(uniforms.light_position - fragment.world_position);
    let view_direction = normalize(uniforms.camera_position - fragment.world_position);
    let height = fragment.local_position.length() - 0.5;

    // Naranja con remolinos claros y poros oscuros
    let warp = fbm(position * 4.0, 3, 6101);
    let swirl = fbm(
        position * 9.0 + Vector3::new(warp * 1.5, -warp, warp * 1.2),
        3,
        6203,
    );
    let lines = 1.0 - smoothstep(0.0, 0.05, (swirl - 0.5).abs());
    let orange = mix(
        ColorF::rgb(0.80, 0.36, 0.05),
        ColorF::rgb(0.97, 0.62, 0.10),
        warp,
    );
    let orange = mix(orange, ColorF::rgb(1.0, 0.80, 0.28), lines * 0.6);
    let pores = smoothstep(0.64, 0.72, fbm(position * 38.0, 2, 6509));
    let orange = mix(orange, ColorF::rgb(0.55, 0.22, 0.03), pores * 0.45);

    // Bordes elevados y bultos: mas claros, como en la referencia
    let ridge = smoothstep(0.004, 0.024, height);
    let orange = mix(orange, ColorF::rgb(1.0, 0.78, 0.38), ridge * 0.55);

    // Cuencas verdes con musgo oscuro, anillos y centro claro
    let green_mask = 1.0 - smoothstep(-0.010, -0.002, height);
    let mound = smoothstep(-0.026, -0.016, height);
    let speckle = fbm(position * 30.0, 2, 6307);
    let moss = smoothstep(0.60, 0.68, fbm(position * 46.0, 2, 6411));
    let green = mix(
        ColorF::rgb(0.08, 0.30, 0.06),
        ColorF::rgb(0.26, 0.52, 0.10),
        speckle,
    );
    let green = mix(green, ColorF::rgb(0.03, 0.15, 0.04), moss * 0.6);
    let green = mix(green, ColorF::rgb(0.80, 0.90, 0.45), mound);
    let surface = mix(orange, green, green_mask);

    // Contorno oscuro en el borde de cada cuenca (estilo caricatura)
    let outline = smoothstep(-0.013, -0.007, height) * (1.0 - smoothstep(-0.004, 0.002, height));
    let surface = mix(surface, ColorF::rgb(0.22, 0.10, 0.02), outline * 0.65);

    // Luz difusa + resplandor rojizo en el borde
    let diffuse = dot(normal, light_direction).max(0.0);
    let glow = (1.0 - dot(normal, view_direction).max(0.0)).powf(3.0);
    (surface * (0.15 + diffuse * 0.85) + ColorF::rgb(0.9, 0.25, 0.25) * (glow * 0.35)).to_color()
}
