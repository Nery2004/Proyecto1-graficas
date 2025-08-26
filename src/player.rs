// player.rs

use raylib::prelude::*;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vector2,
    pub a: f32,
    pub fov: f32, // field of view
}

pub fn process_events(player: &mut Player, rl: &RaylibHandle, dt: f32) {
    // Speeds are per second; movement is frame-rate independent using dt (seconds)
    const MOVE_SPEED: f32 = 120.0; // units per second
    const ROTATION_SPEED: f32 = PI / 2.0; // radians per second (~90 deg/s)

    let move_step = MOVE_SPEED * dt;
    let rot_step = ROTATION_SPEED * dt;

    if rl.is_key_down(KeyboardKey::KEY_LEFT) {
        player.a += rot_step;
    }
    if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
        player.a -= rot_step;
    }
    if rl.is_key_down(KeyboardKey::KEY_DOWN) {
        player.pos.x -= move_step * player.a.cos();
        player.pos.y -= move_step * player.a.sin();
    }
    if rl.is_key_down(KeyboardKey::KEY_UP) {
        player.pos.x += move_step * player.a.cos();
        player.pos.y += move_step * player.a.sin();
    }
}
