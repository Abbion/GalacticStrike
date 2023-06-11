//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use std::collections::btree_map::Range;

use ggez::audio;
use ggez::audio::SoundSource;
use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::{self, Color, Rect};
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
const BULLET_SPEED : f32 = 750.0;

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

fn create_enemies(assets: &Assets) -> Vec<Actor> {
    let mut enemies : Vec<Actor> = Vec::new();

    let mut enemie = create_enemy();
    enemie.size = Vec2{ x: assets.enemie_images[0].width() as f32, y: assets.enemie_images[0].height() as f32 };
    enemies.push(enemie);    

    enemies
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
    hit_sound: audio::Source,
    enemie_images: Vec<graphics::Image>,
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

        let hit_sound = match audio::Source::new(ctx, "/hit.wav") {
            Ok(sound) => sound,
            Err(error) => panic!("Can't load hit shot sound: {:?}", error),
        };

        let mut enemie_images: Vec<graphics::Image> = Vec::with_capacity(3);

        for i in 1..4 {
            let enemie = match graphics::Image::from_path(ctx, format!("/invader{}.png", i)) {
                Ok(image) => image,
                Err(error) => panic!("Can't load enemie {} image: {:?}", i, error),
            };

            enemie_images.push(enemie);
        }

        Assets {
            player_image,
            player_bullet_image,
            player_shot_sound,
            hit_sound,
            enemie_images,
        }
    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Player => &self.player_image,
            ActorType::Bullet => &self.player_bullet_image,
            ActorType::Enemy => &self.enemie_images[0],
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

fn point_in_rect(point: Vec2, rect: Rect) -> bool {

    if point.x > rect.x - rect.w / 2.0 && point.x < rect.w / 2.0 &&
       point.y > rect.y - rect.h / 2.0 && point.y < rect.h / 2.0
    {
       return true; 
    }

    false
}

struct GameState {
    input: InputState,
    assets: Assets,
    player: Actor,
    player_shot_timeout: f32,
    player_bullets: Vec<Actor>,
    enemies: Vec<Actor>,
    window: Window,
}

impl GameState {
    fn new(ctx: &mut Context) -> GameResult<GameState> {
        let (window_width, window_height) = ctx.gfx.drawable_size();
        let assets = Assets::new(ctx);
        let mut player = create_player();

        player.position.y = (window_height / 2.0) - (window_height / 8.0);
        player.size = Vec2{ x: assets.player_image.width() as f32, y: assets.player_image.height() as f32 };
        let enemies = create_enemies(&assets);

        Ok(GameState { 
            input: InputState::default(),
            assets,
            player,
            player_shot_timeout: 0.0,
            player_bullets: Vec::new(),
            enemies: enemies,
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
        self.enemies.retain(|enemie| enemie.hp > 0.0);
    }

    fn handle_collision(&mut self, ctx: &Context) -> GameResult {
        for enemie in &mut self.enemies  {
            if enemie.hp < 0.0 {
                continue;
            }

            let enemie_rect = Rect{ x: enemie.position.x, y: enemie.position.y, w: enemie.size.x, h: enemie.size.y };

            for player_bullet in &mut self.player_bullets {
                if player_bullet.hp < 0.0 {
                    continue;
                }

                let bullet_top = player_bullet.position + Vec2{x: 0.0, y: player_bullet.size.y / 2.0};
                let bullet_down = player_bullet.position - Vec2{x: 0.0, y: player_bullet.size.y / 2.0};
                
                let hit = point_in_rect(bullet_top, enemie_rect) | point_in_rect(bullet_down, enemie_rect);

                if hit {
                    player_bullet.hp = -1.0;
                    enemie.hp = -1.0;
                    self.assets.hit_sound.play(ctx)?;

                    break;
                }
            }
        }
        
        Ok(())
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

            self.handle_collision(ctx)?;
            self.clear_dead_actors();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::from([0.1, 0.2, 0.3, 1.0]));

        let assets = &mut self.assets;
        let world_coords = (self.window.size.x, self.window.size.y);

        let p = &self.player;
        draw_actor(assets, &mut canvas, p, world_coords);

        for bullet in &self.player_bullets {
            draw_actor(assets, &mut canvas, &bullet, world_coords);
        }
        
        for enemie in &self.enemies{
            draw_actor(assets, &mut canvas, enemie, world_coords);
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