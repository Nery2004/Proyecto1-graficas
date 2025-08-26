// caster.rs

use raylib::color::Color;

use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub struct Intersect {
  pub distance: f32,
  pub impact: char,
  pub hit_x: f32,
  pub hit_y: f32,
  pub vertical: bool,
}

pub fn cast_ray(
  framebuffer: &mut Framebuffer,
  maze: &Maze,
  player: &Player,
  a: f32,
  block_size: usize,
  draw_line: bool,
) -> Intersect {
  // DDA raycasting for precise wall hits and correct texture U
  let ray_dir_x = a.cos();
  let ray_dir_y = a.sin();

  // map position (cell indices)
  let mut map_x = (player.pos.x as usize) / block_size;
  let mut map_y = (player.pos.y as usize) / block_size;

  // length of ray from one x or y-side to next x or y-side
  let delta_dist_x = if ray_dir_x.abs() < f32::EPSILON { f32::INFINITY } else { block_size as f32 / ray_dir_x.abs() };
  let delta_dist_y = if ray_dir_y.abs() < f32::EPSILON { f32::INFINITY } else { block_size as f32 / ray_dir_y.abs() };

  let mut step_x: i32 = 0;
  let mut step_y: i32 = 0;
  let mut side_dist_x: f32 = 0.0;
  let mut side_dist_y: f32 = 0.0;

  // calculate step and initial sideDist
  if ray_dir_x < 0.0 {
    step_x = -1;
    side_dist_x = (player.pos.x - (map_x as f32 * block_size as f32)) / ray_dir_x.abs();
  } else {
    step_x = 1;
    side_dist_x = (((map_x + 1) as f32 * block_size as f32) - player.pos.x) / ray_dir_x.abs();
  }
  if ray_dir_y < 0.0 {
    step_y = -1;
    side_dist_y = (player.pos.y - (map_y as f32 * block_size as f32)) / ray_dir_y.abs();
  } else {
    step_y = 1;
    side_dist_y = (((map_y + 1) as f32 * block_size as f32) - player.pos.y) / ray_dir_y.abs();
  }

  // perform DDA
  let mut hit = false;
  let mut side = 0; // 0 = hit vertical (x), 1 = hit horizontal (y)
  let max_steps = 10000; // safety
  let mut steps_taken = 0;

  while !hit && steps_taken < max_steps {
    if side_dist_x < side_dist_y {
      side_dist_x += delta_dist_x;
      map_x = ((map_x as i32) + step_x) as usize;
      side = 0;
    } else {
      side_dist_y += delta_dist_y;
      map_y = ((map_y as i32) + step_y) as usize;
      side = 1;
    }

    // bounds check
    if map_y >= maze.len() || map_x >= maze[0].len() {
      break;
    }

    if maze[map_y][map_x] != ' ' {
      hit = true;
      break;
    }

    steps_taken += 1;
  }

  if !hit {
    // no hit within bounds
    return Intersect { distance: f32::INFINITY, impact: ' ', hit_x: player.pos.x, hit_y: player.pos.y, vertical: false };
  }

  // calculate perpendicular distance using exact wall coordinate (world units)
  let perp_wall_dist = if side == 0 {
    // vertical wall (x side)
    let wall_x = if step_x == 1 { map_x as f32 * block_size as f32 } else { (map_x as f32 + 1.0) * block_size as f32 };
    (wall_x - player.pos.x) / ray_dir_x
  } else {
    // horizontal wall (y side)
    let wall_y = if step_y == 1 { map_y as f32 * block_size as f32 } else { (map_y as f32 + 1.0) * block_size as f32 };
    (wall_y - player.pos.y) / ray_dir_y
  };

  // compute exact hit point in world coordinates
  let hit_x = player.pos.x + perp_wall_dist * ray_dir_x;
  let hit_y = player.pos.y + perp_wall_dist * ray_dir_y;

  // option: draw the cast line (simple sampling)
  if draw_line {
    let dist = ((hit_x - player.pos.x).hypot(hit_y - player.pos.y)) as usize;
    let samples = dist.max(1);
    for s in 0..samples {
      let t = s as f32 / samples as f32;
      let px = (player.pos.x + (hit_x - player.pos.x) * t) as u32;
      let py = (player.pos.y + (hit_y - player.pos.y) * t) as u32;
      framebuffer.set_pixel(px, py);
    }
  }

  let impact = maze[map_y][map_x];
  let vertical_flag = side == 0;

  Intersect { distance: perp_wall_dist, impact, hit_x, hit_y, vertical: vertical_flag }
}