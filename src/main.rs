mod cpu;

extern crate ggez;

use ggez::event;
use ggez::graphics;
use ggez::{Context, GameResult};

const C8_WIDTH: u32 = 640;
const C8_HEIGHT: u32 = 320;

struct MainState {
    pos_x: f32,
    cpu: cpu::Cpu,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        let s = MainState { pos_x: 0.0, cpu: cpu::Cpu::initialize() };
        Ok(s)
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        self.pos_x = self.pos_x % 800.0 + 1.0;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            ggez::nalgebra::Point2::new(0.0, 0.0),
            100.0,
            2.0,
            graphics::WHITE,
        )?;
        graphics::draw(
            ctx,
            &circle,
            (ggez::nalgebra::Point2::new(self.pos_x, 380.0),),
        )?;

        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    use ggez::conf::{WindowMode, WindowSetup};

    let cb = ggez::ContextBuilder::new("Chip8", "ratschance")
        .window_setup(WindowSetup {
            title: "Chip8".to_owned(),
            ..Default::default()
        })
        .window_mode(WindowMode {
            width: C8_WIDTH as f32,
            height: C8_HEIGHT as f32,
            ..Default::default()
        });
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
