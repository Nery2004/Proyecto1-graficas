// framebuffer.rs

use raylib::prelude::*;
use std::f32::consts::PI;

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
    slices_opt: Option<&Vec<(u32, usize, usize, f32, char, f32)>>,
        // minimap sprite texture + minimap positions (in pixels)
        pill_tex_opt: Option<&Texture2D>,
        pill_positions_opt: Option<&Vec<(u32,u32)>>,
    ghost_tex_opt: Option<&Texture2D>,
    ghost_positions_opt: Option<&Vec<(u32,u32)>>,
    ghost_celeste_tex_opt: Option<&Texture2D>,
    ghost_celeste_positions_opt: Option<&Vec<(u32,u32)>>,
        // world-space sprites for 3D projection (positions in world pixels)
    player_pos_opt: Option<&Vector2>,
        player_angle: f32,
        player_fov: f32,
    world_pills_opt: Option<&Vec<Vector2>>,
    world_ghosts_red_opt: Option<&Vec<Vector2>>,
    world_ghosts_celeste_opt: Option<&Vec<Vector2>>,
        // UI values
        fps: i32,
        collected_pills: i32,
        total_pills: i32,
        minimap_cell_size: u32,
    ) {
        if let Ok(texture) = window.load_texture_from_image(raylib_thread, &self.color_buffer) {
            let mut renderer = window.begin_drawing(raylib_thread);
            // Draw the framebuffer first
            renderer.draw_texture(&texture, 0, 0, Color::WHITE);

            // Draw minimap sprites (pills and ghosts) on top of the framebuffer texture
            if let (Some(pill_tex), Some(pill_positions)) = (pill_tex_opt, pill_positions_opt) {
                for (px, py) in pill_positions.iter() {
                    // Dibujar pastilla centrada en la celda
                    let src = Rectangle::new(0.0, 0.0, pill_tex.width as f32, pill_tex.height as f32);
                    let dest = Rectangle::new(
                        *px as f32, 
                        *py as f32, 
                        minimap_cell_size as f32, 
                        minimap_cell_size as f32
                    );
                    renderer.draw_texture_pro(pill_tex, src, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
                }
            }
            if let (Some(ghost_tex), Some(ghost_positions)) = (ghost_tex_opt, ghost_positions_opt) {
                for (px, py) in ghost_positions.iter() {
                    // Dibujar fantasma centrado (rojo)
                    let src = Rectangle::new(0.0, 0.0, ghost_tex.width as f32, ghost_tex.height as f32);
                    let dest = Rectangle::new(
                        *px as f32 - (minimap_cell_size as f32 / 2.0), 
                        *py as f32 - (minimap_cell_size as f32 / 2.0), 
                        minimap_cell_size as f32, 
                        minimap_cell_size as f32
                    );
                    renderer.draw_texture_pro(ghost_tex, src, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
                }
            }
            if let (Some(ghost_tex), Some(ghost_positions)) = (ghost_celeste_tex_opt, ghost_celeste_positions_opt) {
                for (px, py) in ghost_positions.iter() {
                    // Dibujar fantasma centrado (celeste)
                    let src = Rectangle::new(0.0, 0.0, ghost_tex.width as f32, ghost_tex.height as f32);
                    let dest = Rectangle::new(
                        *px as f32 - (minimap_cell_size as f32 / 2.0), 
                        *py as f32 - (minimap_cell_size as f32 / 2.0), 
                        minimap_cell_size as f32, 
                        minimap_cell_size as f32
                    );
                    renderer.draw_texture_pro(ghost_tex, src, dest, Vector2::new(0.0, 0.0), 0.0, Color::WHITE);
                }
            }

            // If we have a wall texture and slices, draw them on top as vertical scaled slices
                    if let (Some(wall_tex), Some(slices)) = (wall_texture_opt, slices_opt) {
                for (x, top, bottom, tex_u, impact, _dist) in slices.iter() {
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

                    // After drawing walls we can project simple world-space sprites (billboards)
                    // to give pills and ghosts a presence in the 3D view.
                    // Requires player info and world sprite lists to be passed in.
                    if let Some(player_pos) = player_pos_opt {
                        let width = self.width as f32;
                        let hh = self.height as f32 / 2.0;
                        // small projection constant that controls overall sprite scale
                        let distance_to_projection_plane = 70.0f32;

                        // helper to normalize angle difference to [-PI,PI]
                        let angle_diff = |a: f32, b: f32| {
                            let mut d = a - b;
                            while d > PI { d -= 2.0 * PI; }
                            while d < -PI { d += 2.0 * PI; }
                            d
                        };

                        // draw pills as small sprites in the 3D world (anchor lower than head)
                        if let (Some(world_pills), Some(pill_tex)) = (world_pills_opt, pill_tex_opt) {
                            for p in world_pills.iter() {
                                let dx = p.x - player_pos.x;
                                let dy = p.y - player_pos.y;
                                let ang = dy.atan2(dx);
                                let rel = angle_diff(ang, player_angle);
                                // skip if outside FOV
                                if rel.abs() > player_fov / 2.0 { continue; }
                                let dist = (dx*dx + dy*dy).sqrt();
                                // perpendicular distance to avoid fish-eye
                                let perp = dist * (rel.cos()).abs().max(0.0001);
                                let current_ray = (rel + (player_fov/2.0)) / player_fov;
                                let sx = current_ray * width;
                                let sprite_h = (hh / perp) * distance_to_projection_plane * 0.7; // pills smaller
                                let sprite_w = sprite_h;
                                // compute depth test against wall slice at same screen x
                                let _sx_i = sx as i32;
                                let mut wall_distance_at_x = std::f32::INFINITY;
                                if let Some(slices) = slices_opt {
                                    for (_x, _top, _bottom, _u, _impact, sdist) in slices.iter() {
                                        // _x is &u32
                                        if (*_x) as f32 == sx.floor() as f32 {
                                            wall_distance_at_x = *sdist;
                                            break;
                                        }
                                    }
                                }
                                // if wall is closer than sprite, skip drawing the sprite
                                if wall_distance_at_x.is_finite() && wall_distance_at_x < dist {
                                    continue;
                                }
                                let src = Rectangle::new(0.0, 0.0, pill_tex.width as f32, pill_tex.height as f32);
                                // anchor sprite so its "feet" appear lower (towards floor)
                                let dest = Rectangle::new(sx - sprite_w/2.0, hh - sprite_h*0.25, sprite_w, sprite_h);
                                renderer.draw_texture_pro(pill_tex, src, dest, Vector2::new(0.0,0.0), 0.0, Color::WHITE);
                            }
                        }

                        // draw ghosts as larger billboards in the 3D world with depth test
                        if let (Some(world_ghosts), Some(ghost_tex)) = (world_ghosts_red_opt, ghost_tex_opt) {
                            for g in world_ghosts.iter() {
                                let dx = g.x - player_pos.x;
                                let dy = g.y - player_pos.y;
                                let ang = dy.atan2(dx);
                                let rel = angle_diff(ang, player_angle);
                                if rel.abs() > player_fov / 2.0 { continue; }
                                let dist = (dx*dx + dy*dy).sqrt();
                                let perp = dist * (rel.cos()).abs().max(0.0001);
                                let current_ray = (rel + (player_fov/2.0)) / player_fov;
                                let sx = current_ray * width;
                                let sprite_h = (hh / perp) * distance_to_projection_plane * 1.4; // ghosts a bit taller
                                let sprite_w = sprite_h;
                                // depth test: check if wall at same x is closer
                                let mut wall_distance_at_x = std::f32::INFINITY;
                                if let Some(slices) = slices_opt {
                                    for (_x, _top, _bottom, _u, _impact, sdist) in slices.iter() {
                                        if (*_x) as f32 == sx.floor() as f32 {
                                            wall_distance_at_x = *sdist;
                                            break;
                                        }
                                    }
                                }
                                if wall_distance_at_x.is_finite() && wall_distance_at_x < dist {
                                    continue; // wall occludes ghost
                                }
                                let src = Rectangle::new(0.0, 0.0, ghost_tex.width as f32, ghost_tex.height as f32);
                                let dest = Rectangle::new(sx - sprite_w/2.0, hh - sprite_h/2.0, sprite_w, sprite_h);
                                renderer.draw_texture_pro(ghost_tex, src, dest, Vector2::new(0.0,0.0), 0.0, Color::WHITE);
                            }
                        }
                        // celeste ghosts
                        if let (Some(world_ghosts), Some(ghost_tex)) = (world_ghosts_celeste_opt, ghost_celeste_tex_opt) {
                            for g in world_ghosts.iter() {
                                let dx = g.x - player_pos.x;
                                let dy = g.y - player_pos.y;
                                let ang = dy.atan2(dx);
                                let rel = angle_diff(ang, player_angle);
                                if rel.abs() > player_fov / 2.0 { continue; }
                                let dist = (dx*dx + dy*dy).sqrt();
                                let perp = dist * (rel.cos()).abs().max(0.0001);
                                let current_ray = (rel + (player_fov/2.0)) / player_fov;
                                let sx = current_ray * width;
                                let sprite_h = (hh / perp) * distance_to_projection_plane * 1.4;
                                let sprite_w = sprite_h;
                                let mut wall_distance_at_x = std::f32::INFINITY;
                                if let Some(slices) = slices_opt {
                                    for (_x, _top, _bottom, _u, _impact, sdist) in slices.iter() {
                                        if (*_x) as f32 == sx.floor() as f32 {
                                            wall_distance_at_x = *sdist;
                                            break;
                                        }
                                    }
                                }
                                if wall_distance_at_x.is_finite() && wall_distance_at_x < dist {
                                    continue; // occluded by wall
                                }
                                let src = Rectangle::new(0.0, 0.0, ghost_tex.width as f32, ghost_tex.height as f32);
                                let dest = Rectangle::new(sx - sprite_w/2.0, hh - sprite_h/2.0, sprite_w, sprite_h);
                                renderer.draw_texture_pro(ghost_tex, src, dest, Vector2::new(0.0,0.0), 0.0, Color::WHITE);
                            }
                        }
                    }
            }

            // Draw readable UI text on top using raylib text instead of the pixel font
            let font_size = 20;
            let padding = 12;
            let fps_s = format!("FPS: {}", fps);
            // place FPS at top-right
            let fps_x = (self.width as i32) - 150;
            let fps_y = padding;
            renderer.draw_text(&fps_s, fps_x, fps_y, font_size, Color::WHITE);

            // draw pill counter at bottom-left
            let counter_s = format!("{}/{}", collected_pills, total_pills);
            let counter_x = padding;
            let counter_y = (self.height as i32) - padding - font_size;
            renderer.draw_text(&counter_s, counter_x, counter_y, font_size, Color::WHITE);
        }
    }
}
