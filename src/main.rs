// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]

mod line;
mod framebuffer;
mod maze;
mod caster;
mod player;

use line::line;
use maze::{Maze,load_maze};
use caster::{cast_ray, Intersect};
use framebuffer::Framebuffer;
use player::{Player, process_events};

use raylib::prelude::*;
use std::thread;
use std::time::{Duration, Instant};
use std::f32::consts::PI;

fn cell_to_color(cell: char) -> Color {
  match cell {
    '+' => {
      return Color::BLUEVIOLET;
    },
    '-' => {
      return Color::VIOLET;
    },
    '|' => {
      return Color::VIOLET;
    },
    'g' => {
      return Color::GREEN;
    },
    _ => {
      return Color::WHITE;
    },
  }
}

fn draw_cell(
  framebuffer: &mut Framebuffer,
  xo: usize,
  yo: usize,
  block_size: usize,
  cell: char,
) {
  if cell == ' ' {
    return;
  }
  let color = cell_to_color(cell);
  framebuffer.set_current_color(color);

  for x in xo..xo + block_size {
    for y in yo..yo + block_size {
      framebuffer.set_pixel(x as u32, y as u32);
    }
  }
}

pub fn render_maze(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  block_size: usize,
  player: &Player,
) {
  for (row_index, row) in maze.iter().enumerate() {
    for (col_index, &cell) in row.iter().enumerate() {
      let xo = col_index * block_size;
      let yo = row_index * block_size;
      draw_cell(framebuffer, xo, yo, block_size, cell);
    }
  }

  framebuffer.set_current_color(Color::WHITESMOKE);

  // draw what the player sees
  let num_rays = 5;
  for i in 0..num_rays {
    let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
    cast_ray(framebuffer, &maze, &player, a, block_size, true);
  }
}

/// Render the 3D world and return optional vertical wall slices as (x, top, bottom)
fn render_world(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  block_size: usize,
  player: &Player,
) -> Vec<(u32, usize, usize, f32, char)> {
  let num_rays = framebuffer.width;

  // let hw = framebuffer.width as f32 / 2.0;   // precalculated half width
  let hh = framebuffer.height as f32 / 2.0;  // precalculated half height

  // draw sky (top half) and floor (bottom half)
  let sky_color = Color::new(3, 6, 46, 255); // #03062e
  let floor_color = Color::BLACK;

  // sky
  framebuffer.set_current_color(sky_color);
  for y in 0..(framebuffer.height / 2) {
    for x in 0..framebuffer.width {
      framebuffer.set_pixel(x, y);
    }
  }

  // floor
  framebuffer.set_current_color(floor_color);
  for y in (framebuffer.height / 2)..framebuffer.height {
    for x in 0..framebuffer.width {
      framebuffer.set_pixel(x, y);
    }
  }

  let mut slices: Vec<(u32, usize, usize, f32, char)> = Vec::new();

  for i in 0..num_rays {
    let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
    let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

    // Calculate the height of the stake
    let distance_to_wall = intersect.distance;// how far is this wall from the player
    let distance_to_projection_plane = 70.0; // how far is the "player" from the "camera"
    // this ratio doesn't really matter as long as it is a function of distance
  let stake_height = (hh / distance_to_wall) * distance_to_projection_plane;
    // Calculate the position to draw the stake
    let stake_top = (hh - (stake_height / 2.0)) as usize;
    let stake_bottom = (hh + (stake_height / 2.0)) as usize;

    // compute texture U coordinate using the vertical flag from the raycaster
    // intersect.hit_x/hit_y are world coords inside the cell
    let local_x = (intersect.hit_x % block_size as f32) / block_size as f32; // 0..1
    let local_y = (intersect.hit_y % block_size as f32) / block_size as f32; // 0..1

    // flip U depending on ray direction to avoid mirrored textures
    let ray_dx = a.cos();
    let ray_dy = a.sin();

    let mut tex_u = if intersect.vertical {
      // vertical face: use fractional x inside the cell
      local_x
    } else {
      // horizontal face: use fractional y inside the cell
      local_y
    };

    if intersect.vertical {
      if ray_dx > 0.0 { tex_u = 1.0 - tex_u; }
    } else {
      if ray_dy < 0.0 { tex_u = 1.0 - tex_u; }
    }

  slices.push((i as u32, stake_top, stake_bottom, tex_u, intersect.impact));
  }

  slices
}

fn render_minimap(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  player: &Player,
  block_size: usize,
  scale: u32,
  margin: u32,
) {
  // compute minimap size
  let maze_w = maze[0].len() as u32;
  let maze_h = maze.len() as u32;
  let map_w = maze_w * scale;
  let map_h = maze_h * scale;

  // top-right corner position
  let x0 = framebuffer.width.saturating_sub(map_w + margin);
  let y0 = margin;

  // draw background box (semi-transparent dark)
  framebuffer.set_current_color(Color::new(20, 20, 20, 200));
  for yy in 0..map_h {
    for xx in 0..map_w {
      framebuffer.set_pixel(x0 + xx, y0 + yy);
    }
  }

  // draw maze cells
  for (j, row) in maze.iter().enumerate() {
    for (i, &cell) in row.iter().enumerate() {
      let color = match cell {
        ' ' => Color::BLACK,
        '+' | '-' | '|' => Color::new(70, 130, 180, 255), // steel blue for walls
        'g' => Color::GREEN,
        _ => Color::WHITE,
      };
      framebuffer.set_current_color(color);
      let px = x0 + (i as u32) * scale;
      let py = y0 + (j as u32) * scale;
      for yy in 0..scale {
        for xx in 0..scale {
          framebuffer.set_pixel(px + xx, py + yy);
        }
      }
    }
  }

  // draw player as a small filled circle and orientation line
  let player_x = (player.pos.x / block_size as f32) * scale as f32; // convert world to minimap pixels
  let player_y = (player.pos.y / block_size as f32) * scale as f32;
  let cx = x0 as f32 + player_x;
  let cy = y0 as f32 + player_y;

  // draw orientation line
  let line_len = scale as f32 * 2.0;
  let lx = cx + player.a.cos() * line_len;
  let ly = cy + player.a.sin() * line_len;
  framebuffer.set_current_color(Color::YELLOW);
  // rasterize a simple Bresenham-like line (integer steps)
  let steps = (line_len * 2.0) as i32;
  for s in 0..steps {
    let t = s as f32 / steps as f32;
    let ix = (cx + (lx - cx) * t) as u32;
    let iy = (cy + (ly - cy) * t) as u32;
    framebuffer.set_pixel(ix, iy);
  }

  // draw player dot
  framebuffer.set_current_color(Color::RED);
  let pr = (scale as f32 / 2.0).max(1.0) as i32;
  for oy in -pr..=pr {
    for ox in -pr..=pr {
      let px = cx as i32 + ox;
      let py = cy as i32 + oy;
      if px >= 0 && py >= 0 {
        framebuffer.set_pixel(px as u32, py as u32);
      }
    }
  }
}

fn main() {
  let window_width = 1300;
  let window_height = 900;
  let block_size = 100;

  let (mut window, raylib_thread) = raylib::init()
    .size(window_width, window_height)
    .title("Raycaster Example")
    .log_level(TraceLogLevel::LOG_WARNING)
    .build();

  let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
  framebuffer.set_background_color(Color::new(50, 50, 100, 255));

  let maze = load_maze("maze.txt");
  let mut player = Player {
    pos: Vector2::new(150.0, 150.0),
    a: PI / 3.0,
    fov: PI / 3.0,
  };

  // Load wall texture from assets
  let wall_texture = window.load_texture(&raylib_thread, "assets/sprites/wall.jpg");
  let wall_texture = match wall_texture {
      Ok(t) => Some(t),
      Err(_) => None,
  };

  // Load portal texture for goal cell 'g'
  let portal_texture = window.load_texture(&raylib_thread, "assets/sprites/portal.jpg");
  let portal_texture = match portal_texture {
    Ok(t) => Some(t),
    Err(_) => None,
  };

  let mut last_frame = Instant::now();

  while !window.window_should_close() {
    // compute delta time (seconds)
    let now = Instant::now();
    let dt = now.duration_since(last_frame).as_secs_f32();
    last_frame = now;
    // 1. clear framebuffer
    framebuffer.clear();

  // 2. move the player on user input (frame-rate independent)
  process_events(&mut player, &window, dt);

    let mut mode = "3D";

    if window.is_key_down(KeyboardKey::KEY_M) {
      mode = if mode == "2D" { "3D" } else { "2D" };
    }

    // 3. draw stuff

  let slices_opt: Option<Vec<(u32, usize, usize, f32, char)>>;

    if mode == "2D" {
      render_maze(&mut framebuffer, &maze, block_size, &player);
      slices_opt = None;
    } else {
      let s = render_world(&mut framebuffer, &maze, block_size, &player);
      slices_opt = Some(s);
    }

  // draw minimap in top-right corner
  // minimap scale: number of pixels per maze cell (increased)
  let minimap_scale: u32 = 10;
  let minimap_margin: u32 = 10;
  render_minimap(&mut framebuffer, &maze, &player, block_size, minimap_scale, minimap_margin);

    // 4. swap buffers - pass wall texture and slices when in 3D
  framebuffer.swap_buffers(&mut window, &raylib_thread, wall_texture.as_ref(), portal_texture.as_ref(), slices_opt.as_ref());

    thread::sleep(Duration::from_millis(16));
  }
}



