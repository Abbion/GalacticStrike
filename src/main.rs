//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use ggez::audio;
use ggez::audio::SoundSource;
use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::{self, Color};
use ggez::input::keyboard::KeyCode;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

#[derive(Debug)]
enum ActorType{
    Player,
    Bullet,
    Enemy,
    Shield,
}

#[derive(Debug)]
struct Actor{
    tag: ActorType,
    position: Vec2,
    direction: Vec2,
    size: Vec2,
    hp: f32,
}

const PLAYER_LIFE : f32 = 1.0;
const BULLET_LIFE : f32 = 1.0;
const ENEMY_LIFE : f32 = 1.0;
const SHIELD_LIFE : f32 = 10.0;
const PLAYER_SPEED : f32 = 320.0;

fn create_player() -> Actor {
    Actor { 
        tag: ActorType::Player,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        hp: PLAYER_LIFE,
     }
}

fn create_bullet() -> Actor {
    Actor { 
        tag: ActorType::Bullet,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        hp: BULLET_LIFE,
     }
}

fn create_enemy() -> Actor {
    Actor{
        tag: ActorType::Enemy,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        hp: ENEMY_LIFE,
    }
}

fn create_shield() -> Actor {
    Actor {
        tag: ActorType::Shield,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        hp: SHIELD_LIFE,
    }
}

fn player_handle_input(actor: &mut Actor, input: &InputState, dt: f32) {
    if input.xaxis != 0.0 {
        actor.direction.x = input.xaxis;
        actor.position.x += input.xaxis * PLAYER_SPEED * dt;
    }
}

#[derive(Debug)]
struct InputState {
    xaxis: f32,
    fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            fire: false,
        }
    }
}

struct Assets {
    player_image: graphics::Image,
}

impl Assets {
    fn new(ctx: &mut Context) -> Assets {
        let player_image = match graphics::Image::from_path(ctx, "/player.png")  {
            Ok(image) => image,
            Err(error) => panic!("Can't load players image: {:?}", error),
        };

        Assets {
            player_image,
        }
    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Player => &self.player_image,
            _ => panic!("No image for: {:?}", actor.tag)
        }
    }
}

fn world_to_screen_coords(screen_width: f32, screen_height: f32, point: Vec2) -> Vec2 {
    let x = point.x + screen_width / 2.0;
    let y = point.y + screen_height / 2.0;
    Vec2::new(x, y)
}

fn draw_actor(assets: &mut Assets, canvas: &mut graphics::Canvas, actor: &Actor, world_coords: (f32, f32)) {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(screen_w, screen_h, actor.position);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam::new()
        .dest(pos)
        .offset(Vec2::new(0.5, 0.5));
    canvas.draw(image, drawparams);
}

struct GameState {
    input: InputState,
    assets: Assets,
    player: Actor,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let assets = Assets::new(ctx);
        let player = create_player();

        Ok(GameState { 
            input: InputState::default(),
            assets,
            player,
         })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const FPS_LIMIT: u32 = 60;

        while ctx.time.check_update_time(FPS_LIMIT) {
            let seconds = 1.0 / (FPS_LIMIT as f32);

            player_handle_input(&mut self.player, &self.input, seconds);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        let assets = &mut self.assets;
        
        let p = &self.player;
        draw_actor(assets, &mut canvas, p, (800.0, 600.0));


        canvas.finish(ctx)?;

        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeated: bool) -> GameResult {
        match input.keycode{
            Some(key) => match key {
                KeyCode::Left => {
                    self.input.xaxis = -1.0;
                },
                KeyCode::Right => {
                    self.input.xaxis = 1.0;
                },
                KeyCode::Space => {
                    if self.input.fire == false {
                        self.input.fire = true;
                    }
                },
                 _ => ()  
            },
            None => ()
        };
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput) -> GameResult {
        match input.keycode{
            Some(key) => match key {
                KeyCode::Left | KeyCode::Right => {
                    self.input.xaxis = 0.0;
                },
                _ => ()
            },
            None => ()
        };

        Ok(())
    }
}

pub fn main() -> GameResult {
    let (mut ctx, events_loop) = ggez::ContextBuilder::new("galactic_strike", "Abbion")
    .window_setup(ggez::conf::WindowSetup::default().title("Galactic strike"))
    .window_mode(ggez::conf::WindowMode::default().dimensions(800.0, 600.0))
    .build()?;
    
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}