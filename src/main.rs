//! The simplest possible example that does something.
#![allow(clippy::unnecessary_wraps)]

use std::collections::btree_map::Range;
use std::vec;
use rand::Rng;

use ggez::audio;
use ggez::audio::SoundSource;
use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::{self, Color, Rect};
use ggez::input::keyboard::KeyCode;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

#[derive(Debug, Copy, Clone)]
enum ActorType {
    Player,
    PlayerBullet,
    EnemyBulletSlow,
    EnemyBulletFast,
    EnemyA,
    EnemyB,
    EnemyC,
    EnemyE,
    Shield,
}

#[derive(PartialEq)]
enum EnemyWallCollisionType {
    Left,
    Right,
    None
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
    scale: Vec2,
    hp: f32,
}
struct EnemiesControler{
    enemies_rect: Rect,
    time_to_update: f32,
    time_to_update_inner: f32,
    tick_time: f32,
    time_to_shoot: f32,
    time_from_last_shot: f32,
    last_collision_type: EnemyWallCollisionType,
    start_to_move: bool,
    row_to_move: u32,
    enemies_alive_flags: Vec<Vec<bool>>,
}

impl Actor {
    fn get_rect(&self) -> Rect {
        Rect{ x: self.position.x - (self.size.x / 2.0), y: self.position.y - (self.size.y / 2.0), w: self.position.x + (self.size.x / 2.0), h: self.position.y + (self.size.y / 2.0) }
    }
}

const PLAYER_LIFE : f32 = 3.0;
const BULLET_LIFE : f32 = 1.0;
const ENEMY_LIFE : f32 = 1.0;
const SHIELD_LIFE : f32 = 10.0;
const PLAYER_SPEED : f32 = 320.0;
const PLAYER_SHOT_TIME : f32 = 0.5;
const PLAYER_BULLET_SPEED : f32 = 750.0;    //750
const ENEMY_BULLET_SPEED_SLOW : f32 = 350.0; //350
const ENEMY_BULLET_SPEED_FAST : f32 = 550.0; //550

const ENEMY_HORIZONTAL_SPACING : f32 = 20.0;
const ENEMY_VERTICAL_SPACING : f32 = 20.0;
const ENEMY_SCALE : f32 = 0.7;
const ENEMY_START_TICK: f32 = 2.0;
const ENEMY_START_SHOT_TIMER: f32 = 3.0;
const ENEMY_JUMP : f32 = 10.0;

fn create_player() -> Actor {
    Actor { 
        tag: ActorType::Player,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        scale: Vec2{ x: 1.0, y: 1.0 },
        hp: PLAYER_LIFE,
     }
}

fn create_bullet(bullet_type: ActorType) -> Actor {
    Actor { 
        tag: bullet_type,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        scale: Vec2{ x: 1.0, y: 1.0 },
        hp: BULLET_LIFE,
     }
}

fn create_enemy() -> Actor {
    Actor{
        tag: ActorType::EnemyA,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        scale: Vec2{ x: 1.0, y: 1.0 },
        hp: ENEMY_LIFE,
    }
}

fn create_shield() -> Actor {
    Actor {
        tag: ActorType::Shield,
        position: Vec2::ZERO,
        direction: Vec2::ZERO,
        size: Vec2::ZERO,
        scale: Vec2{ x: 1.0, y: 1.0 },
        hp: SHIELD_LIFE,
    }
}

fn create_enemies_controler() -> EnemiesControler {
    EnemiesControler { 
        enemies_rect: Rect::zero(),
        time_to_update: 0.0,
        time_to_update_inner: 0.0,
        tick_time: ENEMY_START_TICK,
        time_to_shoot: ENEMY_START_SHOT_TIMER,
        time_from_last_shot: 0.0,
        last_collision_type: EnemyWallCollisionType::None,
        start_to_move: false,
        row_to_move: 5,
        enemies_alive_flags: vec![vec![true; 11]; 5],
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

fn update_bullet_position(actor: &mut Actor, dt: f32) {
    let speed = match actor.tag {
        ActorType::PlayerBullet => PLAYER_BULLET_SPEED,
        ActorType::EnemyBulletSlow => ENEMY_BULLET_SPEED_SLOW,
        ActorType::EnemyBulletFast => ENEMY_BULLET_SPEED_FAST,
        _ => 1.0
    };

    actor.position.y += actor.direction.y * (speed * dt);
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
    let enemy_size = Vec2{ x: assets.enemie_images[0].width() as f32 * ENEMY_SCALE, y: assets.enemie_images[0].height() as f32 * ENEMY_SCALE} ;

    for i in 0..5 {
        let next_position_vertical = (-5.0 * (enemy_size.y + ENEMY_VERTICAL_SPACING)) + (i as f32 * (enemy_size.y + ENEMY_VERTICAL_SPACING));
        let enemy_tag = match i  {
            0 => ActorType::EnemyC,
            1..=2 => ActorType::EnemyB,
            3..=4 => ActorType::EnemyA,
            _ => panic!("No enemy type for this index {}", i)
        }; 

        for j in 0..11 {
            let mut enemie = create_enemy();
            let next_position_horizontal = (-5.0 * (enemy_size.x + ENEMY_HORIZONTAL_SPACING)) + (j as f32 * (enemy_size.x + ENEMY_HORIZONTAL_SPACING));
            
            enemie.tag = enemy_tag;
            enemie.position = Vec2{ x: next_position_horizontal, y: next_position_vertical};
            enemie.size = Vec2{ x: enemy_size.x, y: enemy_size.y };
            enemie.scale = Vec2{ x: ENEMY_SCALE, y: ENEMY_SCALE};
            enemie.direction = Vec2{ x: 1.0, y: 0.0 };
            enemies.push(enemie);
        }
    }

    enemies
}

fn update_enemies_position(enemies_controler: &mut EnemiesControler, enemies: &mut Vec<Actor>, delta_time: f32) 
{
    if enemies.len() < 1 {
        return;
    }

    enemies_controler.time_to_update += delta_time;

    //Check if the enemy move sequence has to start
    if enemies_controler.time_to_update > enemies_controler.tick_time {
        enemies_controler.start_to_move = true;

        //Start counting the inner move times
        enemies_controler.time_to_update_inner += delta_time;
    }

    //Every one fifth of the tick_time move the next row of enemies
    if enemies_controler.time_to_update_inner > enemies_controler.tick_time / 5.0 &&  enemies_controler.start_to_move == true {
        let mut start_end_row_counter = vec![0];
        let mut enemy_index_counter : usize = 0;

        //Calculate start and index of every row of enemies
        for alive_enemies_rows in & enemies_controler.enemies_alive_flags {
            for alive_enemie in alive_enemies_rows {
                if *alive_enemie == true {
                    enemy_index_counter += 1;
                }
            } 

            start_end_row_counter.push(enemy_index_counter);
        }

        let current_row = enemies_controler.row_to_move as usize;

        for (i, enemy) in enemies.iter_mut().skip(start_end_row_counter[current_row - 1]).enumerate() {
            if i + start_end_row_counter[current_row - 1] >= start_end_row_counter[current_row] {
                break;
            }

            enemy.position += enemy.direction * ENEMY_JUMP;
        } 

        //Row that just moved
        enemies_controler.row_to_move -= 1;
        //Inner move timer reset
        enemies_controler.time_to_update_inner = 0.0;

        if enemies_controler.row_to_move <= 0 {
            enemies_controler.row_to_move = start_end_row_counter.len() as u32 - 1;
            enemies_controler.start_to_move = false;
            enemies_controler.time_to_update = 0.0;
        }
    }
}

fn enemies_check_collision_with_walls(enemies_controler: &mut EnemiesControler, enemies: &mut Vec<Actor>, window_size: Vec2)
{
    if enemies_controler.time_to_update == 0.0 && enemies_controler.time_to_update_inner == 0.0 {

        let top_left = Vec2{ x: enemies_controler.enemies_rect.x, y: enemies_controler.enemies_rect.y };
        let bottom_right = Vec2{ x: enemies_controler.enemies_rect.w, y: enemies_controler.enemies_rect.h };

        if bottom_right.x > window_size.x / 2.0 {
            let diff = bottom_right.x - (window_size.x / 2.0);

            for enemy in enemies {
                enemy.direction = Vec2{ x: -1.0, y: 0.0 };
                enemy.position += enemy.direction * diff;
                enemy.direction = Vec2{ x: 0.0, y: 1.0 };
                enemies_controler.last_collision_type = EnemyWallCollisionType::Right;
            }
        }
        else if top_left.x < -window_size.x / 2.0 {
            let diff = -(window_size.x / 2.0) - top_left.x;

            for enemy in enemies {
                enemy.direction = Vec2{ x: 1.0, y: 0.0 };
                enemy.position += enemy.direction * diff;
                enemy.direction = Vec2{ x: 0.0, y: 1.0 };
                enemies_controler.last_collision_type = EnemyWallCollisionType::Left;
            }
        }
        else if enemies_controler.last_collision_type == EnemyWallCollisionType::Right {
            enemies_controler.last_collision_type = EnemyWallCollisionType::None;

            for enemy in enemies {
                enemy.direction = Vec2{ x: -1.0, y: 0.0 };
            }
        }
        else if enemies_controler.last_collision_type == EnemyWallCollisionType::Left {
            enemies_controler.last_collision_type = EnemyWallCollisionType::None;

            for enemy in enemies {
                enemy.direction = Vec2{ x: 1.0, y: 0.0 };
            }
        }
    }
}

fn get_enemies_rect(enemies: &Vec<Actor>) -> Rect
{
    if enemies.len() < 1 {
        return Rect::zero();
    }

    let mut enemies_rect = enemies[0].get_rect();

    for enemy in enemies.iter().skip(1) {
        let enemie_rect = enemy.get_rect();

        if enemie_rect.x < enemies_rect.x {
            enemies_rect.x = enemie_rect.x
        }

        if enemie_rect.y < enemies_rect.y {
            enemies_rect.y = enemie_rect.y
        }

        if enemie_rect.w > enemies_rect.w {
            enemies_rect.w = enemie_rect.w
        }

        if enemie_rect.h > enemies_rect.h {
            enemies_rect.h = enemie_rect.h
        }
    }

    enemies_rect
}

fn enemies_shoot(enemies_controler: &mut EnemiesControler, enemies: &Vec<Actor>, enemy_bullets: &mut Vec<Actor>, delta_time: f32) {
    enemies_controler.time_from_last_shot += delta_time;

    if enemies_controler.start_to_move == false && enemies_controler.time_from_last_shot > enemies_controler.time_to_shoot {
        let mut rng = rand::thread_rng();

        enemies_controler.time_from_last_shot = 0.0;
        enemies_controler.time_to_shoot = rng.gen_range(0.5..3.0);
   
        let mut last_enemies_in_collumn_indexes: Vec<i16> = vec![-1;11];
        
        let mut itr : usize = 0;
        for alive_enemies_rows in enemies_controler.enemies_alive_flags.iter() {
            for (column, alive_enemie) in alive_enemies_rows.iter().enumerate() {
                if *alive_enemie == true {
                    last_enemies_in_collumn_indexes[column] = itr as i16;
                    itr += 1;
                }   
            }
        }

        last_enemies_in_collumn_indexes.retain(|&index| index >= 0);

        let random_enemy = rng.gen_range(0..last_enemies_in_collumn_indexes.len()) as usize;
        let random_enemy = last_enemies_in_collumn_indexes[random_enemy];
        let random_enemy = &enemies[random_enemy as usize];

        let bullet_type = match random_enemy.tag {
            ActorType::EnemyC => ActorType::EnemyBulletFast,
            _ => ActorType::EnemyBulletSlow
        };

        let mut bullet = create_bullet(bullet_type);
        bullet.position = random_enemy.position + Vec2{x: 0.0, y: 35.0};
        bullet.direction.y = 1.0;
        enemy_bullets.push(bullet);
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
    enemy_bullet_slow_image: graphics::Image,
    enemy_bullet_fast_image: graphics::Image,
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

        let enemy_bullet_slow_image = match graphics::Image::from_path(ctx, "/enemy_bullet_slow.png") {
            Ok(image) => image,
            Err(error) => panic!("Can't load enemy bullet slow image: {:?}", error),
        };

        let enemy_bullet_fast_image = match graphics::Image::from_path(ctx, "/enemy_bullet_fast.png") {
            Ok(image) => image,
            Err(error) => panic!("Can't load enemy bullet fast image: {:?}", error),
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
            enemy_bullet_slow_image,
            enemy_bullet_fast_image,
            player_shot_sound,
            hit_sound,
            enemie_images,
        }
    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Player => &self.player_image,
            ActorType::PlayerBullet => &self.player_bullet_image,
            ActorType::EnemyBulletSlow => &self.enemy_bullet_slow_image,
            ActorType::EnemyBulletFast => &self.enemy_bullet_fast_image,
            ActorType::EnemyA => &self.enemie_images[0],
            ActorType::EnemyB => &self.enemie_images[1],
            ActorType::EnemyC => &self.enemie_images[2],
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
        .scale(actor.scale)
        .offset(Vec2::new(0.5, 0.5));

    canvas.draw(image, drawparams);
}

fn point_in_rect(point: &Vec2, rect: &Rect) -> bool {

    if point.x > rect.x && point.x < rect.w &&
       point.y > rect.y && point.y < rect.h
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
    enemy_bullets: Vec<Actor>,
    enemies: Vec<Actor>,
    enemies_controler: EnemiesControler,
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
            enemy_bullets: Vec::new(),
            enemies: enemies,
            enemies_controler: create_enemies_controler(),
            window: Window {
                size : Vec2{ x : window_width, y : window_height }
                }
         })
    }


    fn fire_player_shot(&mut self, ctx: &Context) -> GameResult {
        self.player_shot_timeout = PLAYER_SHOT_TIME;

        let player = &self.player;
        let mut bullet = create_bullet(ActorType::PlayerBullet);
        bullet.position = player.position + Vec2{x: 0.0, y: -10.0};
        bullet.size = Vec2{ x: self.assets.player_bullet_image.width() as f32 * 8.0, y: self.assets.player_bullet_image.height() as f32 };
        bullet.direction.y = -1.0;
        
        self.player_bullets.push(bullet);

        self.assets.player_shot_sound.play(ctx)?;

        Ok(())
    }

    fn clear_dead_actors(&mut self) {
        self.player_bullets.retain(|bullet| bullet.hp > 0.0);
        self.enemy_bullets.retain(|bullet| bullet.hp > 0.0);
        self.enemies.retain(|enemie| enemie.hp > 0.0);
    }

    fn reset_game(&mut self){
        self.player_bullets.clear();
        self.enemy_bullets.clear();
        self.enemies.clear();

        let mut player = create_player();
        player.position.y = (self.window.size.x / 2.0) - (self.window.size.y / 8.0);
        player.size = Vec2{ x: self.assets.player_image.width() as f32, y: self.assets.player_image.height() as f32 };
        let enemies = create_enemies(&self.assets);

        self.player = player;
        self.enemies = enemies;
        self.player_shot_timeout = 0.0;
        self.enemies_controler = create_enemies_controler();
    }

    fn handle_collision(&mut self, ctx: &Context) -> GameResult {
        let mut update_enemies_rect = false;

        for (i, enemie) in &mut self.enemies.iter_mut().enumerate()  {
            if enemie.hp < 0.0 {
                continue;
            }

            let enemie_rect =  enemie.get_rect();

            'outer: for player_bullet in &mut self.player_bullets {
                if player_bullet.hp < 0.0 {
                    continue;
                }

                let bullet_top = player_bullet.position + Vec2{x: 0.0, y: player_bullet.size.y / 2.0};
                let bullet_down = player_bullet.position - Vec2{x: 0.0, y: player_bullet.size.y / 2.0};

                
                let hit = point_in_rect(&bullet_top, &enemie_rect) | point_in_rect(&bullet_down, &enemie_rect);

                if hit {
                    player_bullet.hp = -1.0;
                    enemie.hp = -1.0;
                    self.assets.hit_sound.play(ctx)?;
                    update_enemies_rect = true;
                    self.enemies_controler.tick_time *= 0.8;
                    
                    //Updates the enemies_alive_flags setting false at position where the enemie was destroied
                    let mut index_of_enemy : usize = 0;
                    
                    for alive_enemies_rows in &mut self.enemies_controler.enemies_alive_flags {
                        for alive_enemie in alive_enemies_rows {
                            if *alive_enemie == true {
                                if index_of_enemy == i {
                                    *alive_enemie = false;
                                    break 'outer;
                                }
                                
                                index_of_enemy += 1
                            }
                        }
                    }
                }
            }
        }

        if update_enemies_rect == true {
            self.enemies_controler.enemies_rect = get_enemies_rect(&self.enemies);
        }

        for enemy_bullet in &mut self.enemy_bullets {
            let player_rect = self.player.get_rect();
            let bullet_top = enemy_bullet.position + Vec2{x: 0.0, y: enemy_bullet.size.y / 2.0};
            let bullet_down = enemy_bullet.position - Vec2{x: 0.0, y: enemy_bullet.size.y / 2.0};

            //Hit player
            let hit = point_in_rect(&bullet_top, &player_rect, ) | point_in_rect(&bullet_down, &player_rect);

            if hit {
                enemy_bullet.hp = 0.0;
                self.player.hp -= 1.0;
            }

            //Hit player bullet
            for player_bullet in &mut self.player_bullets {
                let player_bullet_rect = player_bullet.get_rect();
                let bullet_top = enemy_bullet.position + Vec2{x: 0.0, y: enemy_bullet.size.y / 2.0};
                let bullet_down = enemy_bullet.position - Vec2{x: 0.0, y: enemy_bullet.size.y / 2.0};

                let hit = point_in_rect(&bullet_top, &player_bullet_rect, ) | point_in_rect(&bullet_down, &player_bullet_rect);

                if hit {
                    player_bullet.hp = 0.0;
                    enemy_bullet.hp = 0.0;
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
            let delta_time = ctx.time.delta().as_secs_f32();
            
            player_handle_input(&mut self.player, &self.input, delta_time);
            player_check_collision_with_walls(&mut self.player, self.window.size);

            self.player_shot_timeout -= delta_time;
            if self.input.fire && self.player_shot_timeout < 0.0 {
                self.fire_player_shot(ctx)?;
            }

            for act in &mut self.player_bullets {
                update_bullet_position(act, delta_time);
                handle_out_off_screen(act, self.window.size);
            }

            for act in &mut self.enemy_bullets {
                update_bullet_position(act, delta_time);
                handle_out_off_screen(act, self.window.size);
            }

            update_enemies_position(&mut self.enemies_controler, &mut self.enemies, delta_time);
            if self.enemies_controler.time_to_update == 0.0 {
                self.enemies_controler.enemies_rect = get_enemies_rect(&self.enemies);
            }
            enemies_check_collision_with_walls(&mut self.enemies_controler, &mut self.enemies, self.window.size);
            
            enemies_shoot(&mut self.enemies_controler, &self.enemies, &mut self.enemy_bullets, delta_time);

            self.handle_collision(ctx)?;
            self.clear_dead_actors();

            if self.player.hp <= 0.0 {
                self.reset_game();
            }
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

        for bullet in &self.enemy_bullets {
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
    .window_mode(ggez::conf::WindowMode::default().dimensions(650.0, 700.0))
    .build()?;
    
    let state = GameState::new(&mut ctx)?;
    event::run(ctx, events_loop, state)
}