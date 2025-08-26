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
use std::time::Duration;
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
) -> Vec<(u32, usize, usize, f32)> {
  let num_rays = framebuffer.width;

  // let hw = framebuffer.width as f32 / 2.0;   // precalculated half width
  let hh = framebuffer.height as f32 / 2.0;  // precalculated half height

  framebuffer.set_current_color(Color::WHITESMOKE);

  let mut slices: Vec<(u32, usize, usize, f32)> = Vec::new();

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

    // compute texture U coordinate based on impact point inside the cell
    // intersect.impact is the cell char, intersect.hit_x/hit_y are world coords
    let cell_x = (intersect.hit_x as usize) % block_size;
    let cell_y = (intersect.hit_y as usize) % block_size;

    // Decide whether the impact is on a vertical or horizontal face by checking the
    // fractional part closer to edges; a simple heuristic: compare distances to cell borders
    let fx = (intersect.hit_x % block_size as f32) / block_size as f32; // 0..1
    let fy = (intersect.hit_y % block_size as f32) / block_size as f32; // 0..1

    // We'll use x within the cell as tex U for vertical faces, y for horizontal faces
    let tex_u = if fx < 0.0 || fx > 1.0 || fy < 0.0 || fy > 1.0 {
      0.0
    } else if fx.abs() < fy.abs() {
      fx
    } else {
      fy
    };

    slices.push((i as u32, stake_top, stake_bottom, tex_u));
  }

  slices
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

  while !window.window_should_close() {
    // 1. clear framebuffer
    framebuffer.clear();

    // 2. move the player on user input
    process_events(&mut player, &window);

    let mut mode = "3D";

    if window.is_key_down(KeyboardKey::KEY_M) {
      mode = if mode == "2D" { "3D" } else { "2D" };
    }

    // 3. draw stuff

    let slices_opt: Option<Vec<(u32, usize, usize, f32)>>;

    if mode == "2D" {
      render_maze(&mut framebuffer, &maze, block_size, &player);
      slices_opt = None;
    } else {
      let s = render_world(&mut framebuffer, &maze, block_size, &player);
      slices_opt = Some(s);
    }

    // 4. swap buffers - pass wall texture and slices when in 3D
    framebuffer.swap_buffers(&mut window, &raylib_thread, wall_texture.as_ref(), slices_opt.as_ref());

    thread::sleep(Duration::from_millis(16));
  }
}



