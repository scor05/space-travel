use raylib::prelude::{KeyboardKey, RaylibHandle, Vector3};

pub struct Camera {
    pub position: Vector3,
    pub yaw: f32,
    pub field_of_view_y: f32,
    pub near_plane: f32,
    pub far_plane: f32,
    pub movement_speed: f32,
    pub turn_speed: f32,
}

impl Camera {
    pub fn new(position: Vector3) -> Self {
        Self {
            position,
            yaw: 0.0,
            field_of_view_y: 60.0_f32.to_radians(),
            near_plane: 0.1,
            far_plane: 1_000.0,
            movement_speed: 30.0,
            turn_speed: 1.5,
        }
    }

    pub fn update(&mut self, window: &RaylibHandle, delta_time: f32) {
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            self.yaw -= self.turn_speed * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.yaw += self.turn_speed * delta_time;
        }

        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let forward = Vector3::new(sin_yaw, 0.0, cos_yaw);
        let right = Vector3::new(cos_yaw, 0.0, -sin_yaw);
        let mut movement = Vector3::zero();

        if window.is_key_down(KeyboardKey::KEY_W) {
            movement += forward;
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            movement -= forward;
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            movement += right;
        }
        if window.is_key_down(KeyboardKey::KEY_A) {
            movement -= right;
        }

        if movement.length_sqr() > 0.0 {
            let movement_direction = movement * movement.length_sqr().sqrt().recip();
            self.position += movement_direction * self.movement_speed * delta_time;
        }
    }

    // yaw 0 es +Z
    pub fn world_to_view(&self, world_position: Vector3) -> Vector3 {
        let relative = world_position - self.position;
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Vector3::new(
            relative.x * cos_yaw - relative.z * sin_yaw,
            relative.y,
            relative.x * sin_yaw + relative.z * cos_yaw,
        )
    }

    pub fn project(
        &self,
        view_position: Vector3,
        framebuffer_width: u32,
        framebuffer_height: u32,
    ) -> Option<Vector3> {
        if view_position.z <= self.near_plane || view_position.z >= self.far_plane {
            return None;
        }

        let half_height = framebuffer_height as f32 * 0.5;
        let focal_length = half_height / (self.field_of_view_y * 0.5).tan();
        let inverse_z = 1.0 / view_position.z;
        let screen_x = framebuffer_width as f32 * 0.5 + view_position.x * focal_length * inverse_z;
        let screen_y = half_height - view_position.y * focal_length * inverse_z;

        let depth = self.far_plane / (self.far_plane - self.near_plane)
            - (self.far_plane * self.near_plane)
                / ((self.far_plane - self.near_plane) * view_position.z);

        Some(Vector3::new(screen_x, screen_y, depth))
    }
}
