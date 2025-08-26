// framebuffer.rs

use raylib::prelude::*;

pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let color_buffer = Image::gen_image_color(width as i32, height as i32, Color::BLACK);
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color: Color::BLACK,
            current_color: Color::WHITE,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer = Image::gen_image_color(self.width as i32, self.height as i32, self.background_color);
    }

    pub fn set_pixel(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.color_buffer.draw_pixel(x as i32, y as i32, self.current_color);
        }
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn _render_to_file(&self, file_path: &str) {
        self.color_buffer.export_image(file_path);
    }

    /// Swap buffers and optionally draw wall texture slices on top.
    ///
    /// `wall_texture_opt` - optional wall Texture2D to use for vertical slices.
    /// `slices_opt` - optional vector of slices as tuples (x, top, bottom).
    pub fn swap_buffers(
        &self,
        window: &mut RaylibHandle,
        raylib_thread: &RaylibThread,
        wall_texture_opt: Option<&Texture2D>,
        portal_texture_opt: Option<&Texture2D>,
        slices_opt: Option<&Vec<(u32, usize, usize, f32, char)>>,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let mut renderer = window.begin_drawing(raylib_thread);
            // Draw the framebuffer first
            renderer.draw_texture(&texture, 0, 0, Color::WHITE);

            // If we have a wall texture and slices, draw them on top as vertical scaled slices
            if let (Some(wall_tex), Some(slices)) = (wall_texture_opt, slices_opt) {
                for (x, top, bottom, tex_u, impact) in slices.iter() {
                    // skip degenerate slices
                    if bottom <= top {
                        continue;
                    }

                    // choose which texture to draw for this slice (portal or wall)
                    let tex_to_draw: &Texture2D = if *impact == 'g' {
                        match portal_texture_opt {
                            Some(pt) => pt,
                            None => wall_tex,
                        }
                    } else {
                        wall_tex
                    };

                    // source: choose column from the chosen texture based on tex_u (0..1)
                    let tw = tex_to_draw.width as f32;
                    let th = tex_to_draw.height as f32;
                    let mut sx = (tex_u * tw) as f32;
                    if sx < 0.0 { sx = 0.0; }
                    if sx >= tw { sx = tw - 1.0; }

                    let source = Rectangle::new(sx, 0.0, 1.0, th);
                    // destination: scale that column to the stake height on screen
                    let dest = Rectangle::new(*x as f32, *top as f32, 1.0, *bottom as f32 - *top as f32);
                    renderer.draw_texture_pro(tex_to_draw, source, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);

                    // Draw a 1px top border to simulate a ceiling edge for each wall slice
                    let border_color = Color::new(20, 20, 20, 255);
                    renderer.draw_rectangle(*x as i32, *top as i32, 1, 1, border_color);
                }
            }
        }
    }
}
