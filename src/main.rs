#![warn(clippy::all)]
mod cpu;

extern crate ggez;
extern crate rand;

use std::time::{Duration, Instant};

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};


const PIXEL_SIZE: usize = 10;
const SCREEN_WIDTH: usize = cpu::C8_WIDTH * PIXEL_SIZE;
const SCREEN_HEIGHT: usize = cpu::C8_HEIGHT * PIXEL_SIZE;

const MS_PER_UPDATE: u64 = (1000.0 / 60.0) as u64;

struct MainState {
    cpu: cpu::Cpu,
    last_update: Instant,
}

impl MainState {
    fn new(rom: &str) -> GameResult<MainState> {
        let mut s = MainState {
            cpu: cpu::Cpu::initialize(),
            last_update: Instant::now(),
        };
        s.cpu.load_rom(rom);
        s.cpu.load_sprites();
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
        graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
        let display = self.cpu.view_display();

        for i in 0..display.len() {
            for j in 0..display[i].len() {
                if display[i][j] {
                    let rect_bounds = graphics::Rect::new_i32(
                        (j * PIXEL_SIZE) as i32,
                        (i * PIXEL_SIZE) as i32,
                        PIXEL_SIZE as i32,
                        PIXEL_SIZE as i32,
                    );
                    let filled_rect = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        rect_bounds,
                        graphics::WHITE,
                    )?;
                    graphics::draw(ctx, &filled_rect, (ggez::nalgebra::Point2::new(0.0, 0.0),))?;
                }
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }
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
