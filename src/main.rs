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
use rand::random;

#[derive(Clone)]
struct Ghost {
  pos: Vector2,
  dir: Vector2,
  speed: f32,
  kind: char, // 'r' red, 'c' celeste
}

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
) -> Vec<(u32, usize, usize, f32, char, f32)> {
  let num_rays = framebuffer.width;

  // let hw = framebuffer.width as f32 / 2.0;   // precalculated half width
  let hh = framebuffer.height as f32 / 2.0;  // precalculated half height

  // draw sky (top half) and floor (bottom half)
  let sky_color = Color::BLACK;
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

  let mut slices: Vec<(u32, usize, usize, f32, char, f32)> = Vec::new();

  for i in 0..num_rays {
    let current_ray = i as f32 / num_rays as f32; // current ray divided by total rays
    let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
    let intersect = cast_ray(framebuffer, &maze, &player, a, block_size, false);

    // Calculate the height of the stake
  let distance_to_wall = intersect.distance;// how far is this wall from the player
    let distance_to_projection_plane = 70.0; // how far is the "player" from the "camera"
    // ignore invalid or infinite distances
    if !distance_to_wall.is_finite() || distance_to_wall <= 0.0 {
      continue;
    }
    // this ratio doesn't really matter as long as it is a function of distance
    let mut stake_height = (hh / distance_to_wall) * distance_to_projection_plane;
    // clamp stake height so it doesn't become a huge block
    let max_stake = framebuffer.height as f32 * 2.0;
    if stake_height > max_stake { stake_height = max_stake; }

    // Calculate the position to draw the stake and clamp to framebuffer bounds
    let mut stake_top = (hh - (stake_height / 2.0)) as isize;
    let mut stake_bottom = (hh + (stake_height / 2.0)) as isize;
    if stake_top < 0 { stake_top = 0; }
    if stake_bottom > framebuffer.height as isize { stake_bottom = framebuffer.height as isize; }
    if stake_bottom <= stake_top { continue; }
    let stake_top = stake_top as usize;
    let stake_bottom = stake_bottom as usize;

    // compute texture U coordinate using hit point
    let local_x = (intersect.hit_x % block_size as f32) / block_size as f32; // 0..1
    let local_y = (intersect.hit_y % block_size as f32) / block_size as f32; // 0..1

    // some engines use swapped coordinates depending on side; try both mappings for better result
    // here: vertical face (x-side) -> use hit_y; horizontal face (y-side) -> use hit_x
    let mut tex_u = if intersect.vertical {
      local_y
    } else {
      local_x
    };

    // flip based on side direction to avoid mirrored textures
    if intersect.vertical {
      if a.cos() < 0.0 { tex_u = 1.0 - tex_u; }
    } else {
      if a.sin() > 0.0 { tex_u = 1.0 - tex_u; }
    }

  slices.push((i as u32, stake_top, stake_bottom, tex_u, intersect.impact, distance_to_wall));
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
  pill_tex: Option<&Texture2D>,
  ghost_red_tex: Option<&Texture2D>,
) {
  // compute minimap size
  let maze_w = maze[0].len() as u32;
  let maze_h = maze.len() as u32;
  let map_w = maze_w * scale;
  let map_h = maze_h * scale;

  // top-left corner position
  let x0 = margin;
  let y0 = margin;

  // draw background box (semi-transparent dark)
  framebuffer.set_current_color(Color::new(20, 20, 20, 200));
  for yy in 0..map_h {
    for xx in 0..map_w {
      framebuffer.set_pixel(x0 + xx, y0 + yy);
    }
  }

  // draw maze cells and sprites
  for (j, row) in maze.iter().enumerate() {
    for (i, &cell) in row.iter().enumerate() {
      let color = match cell {
        ' ' => Color::BLACK,
        '+' | '-' | '|' => Color::new(70, 130, 180, 255), // steel blue for walls
        'g' => Color::GREEN,
        _ => Color::BLACK,
      };
      framebuffer.set_current_color(color);
      let px = x0 + (i as u32) * scale;
      let py = y0 + (j as u32) * scale;
      for yy in 0..scale {
        for xx in 0..scale {
          framebuffer.set_pixel(px + xx, py + yy);
        }
      }

      // draw sprites on top: '.' = pill, 'r' or 'R' = red ghost
      match cell {
        'o' => {
          if let Some(t) = pill_tex {
            // draw the pill texture scaled to cell
            let _src = Rectangle::new(0.0, 0.0, t.width as f32, t.height as f32);
            let _dest = Rectangle::new(px as f32, py as f32, scale as f32, scale as f32);
            // we cannot draw here with raylib directly; we'll approximate by filling the cell white to represent a pill
            framebuffer.set_current_color(Color::WHITE);
            for yy in 0..scale {
              for xx in 0..scale {
                framebuffer.set_pixel(px + xx, py + yy);
              }
            }
          } else {
            framebuffer.set_current_color(Color::WHITE);
            let cx = px + scale/2;
            let cy = py + scale/2;
            framebuffer.set_pixel(cx, cy);
          }
        }
        'r' | 'R' => {
          if let Some(_t) = ghost_red_tex {
            // draw ghost placeholder using red square (actual texture drawing not possible on Image buffer here)
            framebuffer.set_current_color(Color::RED);
            for yy in 0..scale {
              for xx in 0..scale {
                framebuffer.set_pixel(px + xx, py + yy);
              }
            }
          } else {
            framebuffer.set_current_color(Color::RED);
            let cx = px + scale/2;
            let cy = py + scale/2;
            framebuffer.set_pixel(cx, cy);
          }
        }
        _ => {}
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
    .title("Pacman Horror Game ")
    .log_level(TraceLogLevel::LOG_WARNING)
    .build();

  let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
  framebuffer.set_background_color(Color::new(50, 50, 100, 255));

  let mut maze = load_maze("maze.txt");
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

  // count total pills present in the maze (cells with '.')

  // extract pills from maze into an entity list so they don't act as walls
  let mut pills: Vec<Vector2> = Vec::new();
  for j in 0..maze.len() {
    for i in 0..maze[j].len() {
      if maze[j][i] == '.' {
        let wx = (i as f32 + 0.5) * block_size as f32;
        let wy = (j as f32 + 0.5) * block_size as f32;
        pills.push(Vector2::new(wx, wy));
        // clear from maze so it doesn't block movement or render as a wall
        maze[j][i] = ' ';
      }
    }
  }

  let mut total_pills: i32 = pills.len() as i32;
  let mut collected_pills: i32 = 0;

  // extract initial ghosts from the maze and track them as entities
  let mut ghosts: Vec<Ghost> = Vec::new();
  // iterate rows and columns using each row's length so we don't index past ragged rows
  for j in 0..maze.len() {
    for i in 0..maze[j].len() {
      let cell = maze[j][i];
      if cell == 'r' || cell == 'R' {
        let gx = (i as f32 + 0.5) * block_size as f32;
        let gy = (j as f32 + 0.5) * block_size as f32;
        ghosts.push(Ghost { pos: Vector2::new(gx, gy), dir: Vector2::new(1.0, 0.0), speed: 50.0, kind: 'r' });
        // clear marker so minimap draws ghost from entity list only
        maze[j][i] = ' ';
      }
      if cell == 'c' || cell == 'C' {
        let gx = (i as f32 + 0.5) * block_size as f32;
        let gy = (j as f32 + 0.5) * block_size as f32;
        ghosts.push(Ghost { pos: Vector2::new(gx, gy), dir: Vector2::new(1.0, 0.0), speed: 50.0, kind: 'c' });
        maze[j][i] = ' ';
      }
    }
  }

  let mut last_frame = Instant::now();

  // load sprite textures for minimap
  let sprites_pastillas = window.load_texture(&raylib_thread, "assets/sprites/pastillas.png");
  let sprites_pastillas = match sprites_pastillas { Ok(t) => Some(t), Err(_) => None };
  let sprite_fantasma_rojo = window.load_texture(&raylib_thread, "assets/sprites/fantasma_rojo.png");
  let sprite_fantasma_rojo = match sprite_fantasma_rojo { Ok(t) => Some(t), Err(_) => None };
  let sprite_fantasma_celeste = window.load_texture(&raylib_thread, "assets/sprites/fantasma_celeste.png");
  let sprite_fantasma_celeste = match sprite_fantasma_celeste { Ok(t) => Some(t), Err(_) => None };

  while !window.window_should_close() {
    // compute delta time (seconds)
    let now = Instant::now();
    let dt = now.duration_since(last_frame).as_secs_f32();
    last_frame = now;
    // 1. clear framebuffer
    framebuffer.clear();

  // 2. move the player on user input (frame-rate independent) and resolve circular collision
  // We'll apply movement first, then push the player out of nearby wall cells if overlapping.
  let mut candidate = player.pos;
  // copy movement logic from process_events but write into candidate
  const MOVE_SPEED: f32 = 120.0;
  const ROTATION_SPEED: f32 = std::f32::consts::PI / 2.0;
  let move_step = MOVE_SPEED * dt;
  let rot_step = ROTATION_SPEED * dt;
  if window.is_key_down(KeyboardKey::KEY_LEFT) {
    player.a += rot_step;
  }
  if window.is_key_down(KeyboardKey::KEY_RIGHT) {
    player.a -= rot_step;
  }
  if window.is_key_down(KeyboardKey::KEY_DOWN) {
    candidate.x -= move_step * player.a.cos();
    candidate.y -= move_step * player.a.sin();
  }
  if window.is_key_down(KeyboardKey::KEY_UP) {
    candidate.x += move_step * player.a.cos();
    candidate.y += move_step * player.a.sin();
  }

  // circular collision parameters
  let player_radius: f32 = 16.0; // pixels
  // check nearby cells within radius cells to resolve collisions
  let r_cells = ((player_radius / block_size as f32).ceil() as isize) + 1;
  let pi = (candidate.x as isize) / (block_size as isize);
  let pj = (candidate.y as isize) / (block_size as isize);

  // iterate nearby wall cells and push candidate out if overlapping
  let mut resolved = candidate;
  for oy in -r_cells..=r_cells {
    for ox in -r_cells..=r_cells {
      let cx = pi + ox;
      let cy = pj + oy;
      if cx < 0 || cy < 0 { continue; }
  let ux = cx as usize;
  let uy = cy as usize;
  if uy >= maze.len() { continue; }
  if ux >= maze[uy].len() { continue; }
  if maze[uy][ux] == ' ' { continue; }

      // compute nearest point on cell box to candidate position
      let cell_min_x = ux as f32 * block_size as f32;
      let cell_min_y = uy as f32 * block_size as f32;
      let cell_max_x = cell_min_x + block_size as f32;
      let cell_max_y = cell_min_y + block_size as f32;

      // clamped point
      let nearest_x = resolved.x.max(cell_min_x).min(cell_max_x);
      let nearest_y = resolved.y.max(cell_min_y).min(cell_max_y);
      let dx = resolved.x - nearest_x;
      let dy = resolved.y - nearest_y;
      let dist2 = dx*dx + dy*dy;
      let min_dist = player_radius;
      if dist2 < (min_dist * min_dist) {
        let dist = dist2.sqrt().max(0.001);
        // push out along vector from nearest point to player center
        let push_x = dx / dist * (min_dist - dist);
        let push_y = dy / dist * (min_dist - dist);
        resolved.x += push_x;
        resolved.y += push_y;
      }
    }
  }

  // apply resolved position
  player.pos = resolved;

  // check pill pickup by distance to pill entities (player radius)
  let pickup_radius = 24.0;
  let mut remove_indexes: Vec<usize> = Vec::new();
  for (idx, p) in pills.iter().enumerate() {
    let dx = p.x - player.pos.x;
    let dy = p.y - player.pos.y;
    if dx*dx + dy*dy <= pickup_radius * pickup_radius {
      remove_indexes.push(idx);
    }
  }
  // remove collected pills (in reverse to keep indexes valid)
  for &ri in remove_indexes.iter().rev() {
    pills.remove(ri);
    collected_pills += 1;
  }

    let mut mode = "3D";

    if window.is_key_down(KeyboardKey::KEY_M) {
      mode = if mode == "2D" { "3D" } else { "2D" };
    }

    // 3. draw stuff

  let slices_opt: Option<Vec<(u32, usize, usize, f32, char, f32)>>;

    if mode == "2D" {
      render_maze(&mut framebuffer, &maze, block_size, &player);
      slices_opt = None;
    } else {
      let s = render_world(&mut framebuffer, &maze, block_size, &player);
        slices_opt = Some(s);
    }

  // draw minimap in top-left corner only when player is close to a wall
  // make minimap bigger
  let minimap_scale: u32 = 12;
  let minimap_margin: u32 = 10;

  // check nearby cells within radius (in cells) to decide whether to show minimap
  let show_minimap = {
    let radius = 2usize; // cells
    let pi = (player.pos.x as isize) / (block_size as isize);
    let pj = (player.pos.y as isize) / (block_size as isize);
    let mut near = false;
    for oy in -(radius as isize)..=(radius as isize) {
      for ox in -(radius as isize)..=(radius as isize) {
        let x = pi + ox;
        let y = pj + oy;
        if x >= 0 && y >= 0 && (y as usize) < maze.len() {
          let ry = y as usize;
          let rx = x as usize;
          if rx < maze[ry].len() && maze[ry][rx] != ' ' {
            near = true;
            break;
          }
        }
      }
      if near { break; }
    }
    near
  };

  if show_minimap {
    render_minimap(&mut framebuffer, &maze, &player, block_size, minimap_scale, minimap_margin, sprites_pastillas.as_ref(), sprite_fantasma_rojo.as_ref());
  }
  // compute fps
  let fps = if dt > 0.0 { (1.0 / dt).round() as i32 } else { 0 };

  // 3.5 move ghosts: simple AI - if within chase_radius chase player, else wander
  let chase_radius = 300.0;
  for ghost in ghosts.iter_mut() {
    let to_player = player.pos - ghost.pos;
    let dist = to_player.length();
    let mut desired = ghost.dir;
    if dist < chase_radius {
      // chase
      if to_player.length() > 0.0 {
        let mut tp = to_player;
        tp.normalize();
        desired = tp;
      } else {
        desired = Vector2::new(0.0, 0.0);
      }
    } else {
      // wander slowly: small random perturbation
      let wobble = 0.5;
      desired.x += (rand::random::<f32>() - 0.5) * wobble;
      desired.y += (rand::random::<f32>() - 0.5) * wobble;
      if desired.length() == 0.0 {
        desired = Vector2::new(1.0, 0.0);
      } else {
        let mut d = desired;
        d.normalize();
        desired = d;
      }
    }
    ghost.dir = desired;
    // attempt movement
    let move_step = ghost.dir * ghost.speed * dt;
  let candidate = ghost.pos + move_step;
    // ghost collision vs walls: prevent entering wall cells using simple AABB test
    let gx_cell = (candidate.x as usize) / block_size;
    let gy_cell = (candidate.y as usize) / block_size;
    if gy_cell < maze.len() {
      if gx_cell < maze[gy_cell].len() {
        if maze[gy_cell][gx_cell] == ' ' {
          ghost.pos = candidate;
        } else {
          // hit wall: bounce direction
          ghost.dir = Vector2::new(-ghost.dir.x, -ghost.dir.y);
        }
      }
    }
  }

  // detect ghost-player collision (simple distance check)
  let mut player_hit = false;
  for ghost in ghosts.iter() {
    let d2 = (ghost.pos.x - player.pos.x)*(ghost.pos.x - player.pos.x) + (ghost.pos.y - player.pos.y)*(ghost.pos.y - player.pos.y);
    let min_dist = 20.0 + 16.0; // ghost radius + player radius
    if d2 < min_dist*min_dist {
      player_hit = true;
      break;
    }
  }
  if player_hit {
    // simple response: reset player position to start
    player.pos = Vector2::new(150.0, 150.0);
  }

// 4. swap buffers - pass wall texture and slices when in 3D
// prepare pill positions (from maze) and ghost positions (from entities) for minimap
let mut pill_positions: Vec<(u32,u32)> = Vec::new();
let mut ghost_positions: Vec<(u32,u32)> = Vec::new();
let map_x0 = minimap_margin;
let map_y0 = minimap_margin;

// PILL POSITIONS - from pill entities (centers) for minimap
for p in pills.iter() {
  let gx = map_x0 as f32 + (p.x / block_size as f32) * minimap_scale as f32;
  let gy = map_y0 as f32 + (p.y / block_size as f32) * minimap_scale as f32;
  pill_positions.push((gx as u32, gy as u32));
}

// GHOST POSITIONS for minimap: separate red and celeste
let mut ghost_positions_red: Vec<(u32,u32)> = Vec::new();
let mut ghost_positions_celeste: Vec<(u32,u32)> = Vec::new();
for g in ghosts.iter() {
  let gx = map_x0 as f32 + (g.pos.x / block_size as f32) * minimap_scale as f32;
  let gy = map_y0 as f32 + (g.pos.y / block_size as f32) * minimap_scale as f32;
  if g.kind == 'r' { ghost_positions_red.push((gx as u32, gy as u32)); }
  else if g.kind == 'c' { ghost_positions_celeste.push((gx as u32, gy as u32)); }
}

// World-space lists for 3D sprite projection (separate)
let world_pills: Vec<Vector2> = pills.iter().map(|p| Vector2::new(p.x, p.y)).collect();
let world_ghosts_red: Vec<Vector2> = ghosts.iter().filter(|g| g.kind == 'r').map(|g| Vector2::new(g.pos.x, g.pos.y)).collect();
let world_ghosts_celeste: Vec<Vector2> = ghosts.iter().filter(|g| g.kind == 'c').map(|g| Vector2::new(g.pos.x, g.pos.y)).collect();

framebuffer.swap_buffers(&mut window, &raylib_thread, wall_texture.as_ref(), portal_texture.as_ref(), slices_opt.as_ref(), sprites_pastillas.as_ref(), Some(&pill_positions), sprite_fantasma_rojo.as_ref(), Some(&ghost_positions_red), sprite_fantasma_celeste.as_ref(), Some(&ghost_positions_celeste), Some(&player.pos), player.a, player.fov, Some(&world_pills), Some(&world_ghosts_red), Some(&world_ghosts_celeste), fps, collected_pills, total_pills, minimap_scale);

    thread::sleep(Duration::from_millis(16));
  }
}



