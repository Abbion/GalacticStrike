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
struct Window {
    size : Vec2, 
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
const PLAYER_SHOT_TIME : f32 = 0.5;
const BULLET_SPEED : f32 = 700.0;

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
    actor.direction.x = 0.0;

    if input.left {
        actor.direction.x += -1.0;
    }
    if input.right {
        actor.direction.x += 1.0;
    }

    actor.position.x += actor.direction.x * PLAYER_SPEED * dt;
}

fn player_check_collision_with_walls(actor: &mut Actor, window_size: Vec2) {
    let left_edge = actor.position.x - (actor.size.x / 2.0);
    let right_edge = actor.position.x + (actor.size.x / 2.0);

    if left_edge < -window_size.x / 2.0 {
        actor.position.x = -window_size.x / 2.0 + actor.size.x / 2.0;
    }
    else if right_edge > window_size.x / 2.0 {
        actor.position.x = (window_size.x / 2.0) - (actor.size.x / 2.0);
    }
}

fn update_actor_position(actor: &mut Actor, dt: f32) {
    actor.position.y += actor.direction.y * (BULLET_SPEED * dt);
}

fn handle_out_off_screen(actor: &mut Actor, window_size: Vec2) {
    if  actor.position.y < -window_size.y / 2.0 ||
        actor.position.y > window_size.y / 2.0 ||
        actor.position.x < -window_size.x / 2.0 ||
        actor.position.x > window_size.x / 2.0 {
        actor.hp = -1.0;
    }
}

#[derive(Debug)]
struct InputState {
    left: bool,
    right: bool,
    fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            left: false,
            right: false,
            fire: false,
        }
    }
}

struct Assets {
    player_image: graphics::Image,
    player_bullet_image: graphics::Image,
    player_shot_sound: audio::Source,
}

impl Assets {
    fn new(ctx: &mut Context) -> Assets {
        let player_image = match graphics::Image::from_path(ctx, "/player.png")  {
            Ok(image) => image,
            Err(error) => panic!("Can't load player image: {:?}", error),
        };

        let player_bullet_image = match graphics::Image::from_path(ctx, "/player_bullet.png") {
            Ok(image) => image,
            Err(error) => panic!("Can't load player bullet image: {:?}", error),
        };

        let player_shot_sound = match audio::Source::new(ctx, "/player_shoot_sound.wav") {
            Ok(sound) => sound,
            Err(error) => panic!("Can't load player shot sound: {:?}", error),
        };

        Assets {
            player_image,
            player_bullet_image,
            player_shot_sound,
        }
    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Player => &self.player_image,
            ActorType::Bullet => &self.player_bullet_image,
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
    player_shot_timeout: f32,
    player_bullets: Vec<Actor>,
    window: Window,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let (window_width, window_height) = ctx.gfx.drawable_size();
        let assets = Assets::new(ctx);
        let mut player = create_player();

        player.position.y = (window_height / 2.0) - (window_height / 8.0);
        player.size = Vec2{ x: assets.player_image.width() as f32, y: assets.player_image.height() as f32 };


        Ok(GameState { 
            input: InputState::default(),
            assets,
            player,
            player_shot_timeout: 0.0,
            player_bullets: Vec::new(),
            window: Window {
                size : Vec2{ x : window_width, y : window_height }
                }
         })
    }

    fn fire_player_shot(&mut self, ctx: &Context) -> GameResult {
        self.player_shot_timeout = PLAYER_SHOT_TIME;

        let player = &self.player;
        let mut bullet = create_bullet();
        bullet.position = player.position + Vec2{x: 0.0, y: -10.0};
        bullet.direction.y = -1.0;
        
        self.player_bullets.push(bullet);

        self.assets.player_shot_sound.play(ctx)?;

        Ok(())
    }

    fn clear_dead_actors(&mut self) {
        self.player_bullets.retain(|bullet| bullet.hp > 0.0);
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const FPS_LIMIT: u32 = 60;

        while ctx.time.check_update_time(FPS_LIMIT) {
            let seconds = 1.0 / (FPS_LIMIT as f32);

            player_handle_input(&mut self.player, &self.input, seconds);
            player_check_collision_with_walls(&mut self.player, self.window.size);

            self.player_shot_timeout -= seconds;
            if self.input.fire && self.player_shot_timeout < 0.0 {
                self.fire_player_shot(ctx)?;
            }

            for act in &mut self.player_bullets {
                update_actor_position(act, seconds);
                handle_out_off_screen(act, self.window.size);
            }

            self.clear_dead_actors();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        let assets = &mut self.assets;
        
        let p = &self.player;
        draw_actor(assets, &mut canvas, p, (self.window.size.x, self.window.size.y));

        for bullet in &self.player_bullets {
            draw_actor(assets, &mut canvas, &bullet, (self.window.size.x, self.window.size.y));
        }


        canvas.finish(ctx)?;

        timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: ggez::input::keyboard::KeyInput, _repeated: bool) -> GameResult {
        match input.keycode{
            Some(key) => match key {
                KeyCode::Left => {
                    self.input.left = true;
                },
                KeyCode::Right => {
                    self.input.right = true;
                },
                KeyCode::Space => {
                    self.input.fire = true;
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
                KeyCode::Left => {
                    self.input.left = false;
                },
                KeyCode::Right => {
                    self.input.right = false;
                },
                KeyCode::Space => {
                    self.input.fire = false;
                }
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