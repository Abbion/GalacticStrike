//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use ggez::{
    event,
    glam::*,
    graphics::{self},
    Context, GameResult, winit::event::VirtualKeyCode,
};

struct GameState {
    
}

impl GameState {
    fn new() -> GameResult<GameState> {
        Ok(GameState {})
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        canvas.finish(ctx)?;

        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeated: bool) -> GameResult {
        match input.keycode{
            Some(key) => match key {
                VirtualKeyCode::Left => println!("<=="),
                VirtualKeyCode::Right => println!("==>"),
                VirtualKeyCode::Space => println!("Pow!"),
                 _ => ()  
            },
            None => ()
        };

        Ok(())
    }
}

pub fn main() -> GameResult {
    let (ctx, events_loop) = ggez::ContextBuilder::new("galactic_strike", "Abbion")
    .window_setup(ggez::conf::WindowSetup::default().title("Galactic strike"))
    .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0))
    .build()?;
    
    let state = GameState::new()?;
    event::run(ctx, events_loop, state)
}