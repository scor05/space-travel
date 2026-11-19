use raylib::prelude::*;

// current_color sirve para pintar varios pixeles a la vez del mismo color
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    emissive_buffer: Image,
    pub background_color: Color,
    pub current_color: Color,
    depth_buffer: Vec<f32>,
}

/*
 * Para dibujar:
 *  - Escuchar input (usuario)
 *  - Limpiar frame anterior
 *  - Preparar variables / actualizarlas
 *  - Pintar al buffer
 *  - Pintar el buffer a la ventana (swap/flip buffers)
 *  - Esperar (vsync)
 * */
impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        let emissive_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLANK);
        Framebuffer {
            width,
            height,
            color_buffer,
            emissive_buffer,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
            depth_buffer: vec![f32::INFINITY; (width * height) as usize],
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer.clear_background(self.background_color);
        self.emissive_buffer.clear_background(Color::BLANK);
        self.depth_buffer.fill(f32::INFINITY);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer
                .draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_pixel_color(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, color);
        }
    }

    pub fn shade_fragment<F>(&mut self, x: i32, y: i32, depth: f32, emissive: bool, shade: F)
    where
        F: FnOnce() -> Color,
    {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }

        let index = y as usize * self.width as usize + x as usize;
        if depth < self.depth_buffer[index] {
            let color = shade();
            self.depth_buffer[index] = depth;
            self.color_buffer.draw_pixel(x, y, color);
            if emissive {
                self.emissive_buffer.draw_pixel(x, y, color);
            } else {
                self.emissive_buffer.draw_pixel(x, y, Color::BLANK);
            }
        }
    }

    pub fn apply_bloom(&mut self) {
        let original = self.emissive_buffer.clone();
        let mut wide_glow = original.clone();
        let mut near_glow = original.clone();
        wide_glow.blur_gaussian(75);
        near_glow.blur_gaussian(30);

        let bounds = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);
        self.color_buffer
            .draw(&wide_glow, bounds, bounds, Color::new(45, 105, 255, 150));
        self.color_buffer
            .draw(&near_glow, bounds, bounds, Color::new(105, 190, 255, 185));
        self.color_buffer
            .draw(&original, bounds, bounds, Color::WHITE);
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn color_bytes(&self) -> &[u8] {
        let byte_count = self.color_buffer.get_pixel_data_size();

        unsafe { std::slice::from_raw_parts(self.color_buffer.data() as *const u8, byte_count) }
    }

    pub fn swap_buffers(
        &mut self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        display_texture: &mut Texture2D,
    ) {
        display_texture
            .update_texture(self.color_bytes())
            .expect("Error on framebuffer image update");

        let win_height = window.get_render_height();
        let win_width = window.get_render_width();

        let mut renderer = window.begin_drawing(raylib_thread);

        // escalar pixeles si framebuffer es más pequeño que la ventana
        let source = Rectangle::new(0.0, 0.0, self.width as f32, self.height as f32);
        let dest = Rectangle::new(0.0, 0.0, win_width as f32, win_height as f32);

        /*
         * parámetros de pro:
         * la textura
         * source_rect: parte de la textura que usa (crop)
         * dest_rect: tamaño a lo que se escalará el source_rect
         * origin: pivote para posicionar/rotar dest_rect
         * rotation
         * tint: color que se le multiplica la imagen
         */
        renderer.draw_texture_pro(
            &display_texture,
            source,
            dest,
            Vector2::new(0.0, 0.0),
            0.0,
            Color::WHITE,
        );
    }
}
