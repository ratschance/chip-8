#![warn(clippy::all)]
mod cpu;

extern crate ggez;
extern crate rand;

use std::time::{Duration, Instant};

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics;
use ggez::{Context, GameResult};

const PIXEL_SIZE: usize = 10;
const SCREEN_WIDTH: usize = cpu::C8_WIDTH * PIXEL_SIZE;
const SCREEN_HEIGHT: usize = cpu::C8_HEIGHT * PIXEL_SIZE;

const MS_PER_UPDATE: u64 = 2 as u64; // 500hz suggested cycle rate

struct MainState {
    cpu: cpu::Cpu,
    last_update: Instant,
    // Keep last three frames to smooth animation by taking the logical or of each pixel
    last_frames: [[[bool; cpu::C8_WIDTH]; cpu::C8_HEIGHT]; 3],
}

impl MainState {

    /// Creates a new MainState, initializes the CPU and loads a ROM based on the passed filepath
    ///
    /// # Arguments
    ///
    /// * `rom` - Path to ROM file. Will panic if file does not exist
    fn new(rom: &str) -> GameResult<MainState> {
        let mut s = MainState {
            cpu: cpu::Cpu::initialize(),
            last_update: Instant::now(),
            last_frames: [[[false; cpu::C8_WIDTH]; cpu::C8_HEIGHT]; 3]
        };
        s.cpu.load_rom(rom);
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if Instant::now() - self.last_update >= Duration::from_millis(MS_PER_UPDATE) {
            self.last_update = Instant::now();
            self.cpu.tick();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if self.cpu.has_disp_update() {
            graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
            self.last_frames[2].copy_from_slice(self.cpu.view_display());
            let rect_bounds = graphics::Rect::new_i32(0, 0, PIXEL_SIZE as i32, PIXEL_SIZE as i32);
            let filled_rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                rect_bounds,
                graphics::WHITE,
            )?;

            for i in 0..cpu::C8_HEIGHT {
                for j in 0..cpu::C8_WIDTH {
                    if self.last_frames[0][i][j] | self.last_frames[1][i][j] | self.last_frames[2][i][j]{
                        graphics::draw(
                            ctx,
                            &filled_rect,
                            (ggez::nalgebra::Point2::new(
                                (j * PIXEL_SIZE) as f32,
                                (i * PIXEL_SIZE) as f32,
                            ),),
                        )?;
                    }
                }
            }

            graphics::present(ctx)?;
        } else {
            self.last_frames[0] = self.last_frames[1];
            self.last_frames[1] = self.last_frames[2];
        }
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        if let Some(idx) = get_idx_from_keycode(keycode) {
            self.cpu.set_key_pressed(idx);
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        if let Some(idx) = get_idx_from_keycode(keycode) {
            self.cpu.set_key_released(idx)
        }
    }
}

/// Map keyboard keys to Chip-8 keys
///
///  1 2 3 4    1 2 3 C
///  q w e r -> 4 5 6 D
///  a s d f    7 8 9 E
///  z x c v    A 0 B F
fn get_idx_from_keycode(keycode: KeyCode) -> Option<usize> {
    let key = match keycode {
        KeyCode::Key1 => 1,
        KeyCode::Key2 => 2,
        KeyCode::Key3 => 3,
        KeyCode::Key4 => 0xC,
        KeyCode::Q => 4,
        KeyCode::W => 5,
        KeyCode::E => 6,
        KeyCode::R => 0xD,
        KeyCode::A => 7,
        KeyCode::S => 8,
        KeyCode::D => 9,
        KeyCode::F => 0xE,
        KeyCode::Z => 0xA,
        KeyCode::X => 0,
        KeyCode::C => 0xB,
        KeyCode::V => 0xF,
        _ => return None,
    };
    Some(key)
}

fn main() -> GameResult {
    use ggez::conf::{WindowMode, WindowSetup};

    let args: Vec<String> = std::env::args().collect();
    let cb = ggez::ContextBuilder::new("Chip8", "ratschance")
        .window_setup(WindowSetup {
            title: "Chip8".to_owned(),
            ..Default::default()
        })
        .window_mode(WindowMode {
            width: SCREEN_WIDTH as f32,
            height: SCREEN_HEIGHT as f32,
            ..Default::default()
        });
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(&args[1])?;
    event::run(ctx, event_loop, state)
}
