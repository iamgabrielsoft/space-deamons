
use macroquad::prelude::*;
use quad_snd::mixer::{SoundMixer, Volume}; 
use rand::rand; 



use crate::{
    constants::*, 
    resources::{Resources, SoundIdentifier},
    wave::{ WaveManager, LastEnermyDeathReason},
    bullet:: { Bullet, BulletHurtType }
};



#[derive(Clone, Copy)]
pub enum EnermyType {
    NORMAL, 
    MINI
}


#[derive(Clone, Copy)]
pub enum EnermyColor {
    PURPLE, 
    RED, 
    GREEN
}

pub enum EnermyCommand {
    ChangeState(EnermyState),
}



impl EnermyColor {
    pub fn random() -> Self {
        use EnermyColor::*; 
        let all = [PURPLE, GREEN, RED];
        return all[rand::gen_range(0, all.len())]
    }
}


pub struct EnermyStateShared {
    texture: Texture2D, 
    angle:f32, 
    angle_speed: f32, 
    collision_rect: Rect, 
    pub health: i32, 
    pub death_method: EnermyDeathMethod, 
    animation_timer: f32, 
    pub enermy_type: EnermyType, 
    pub enermy_color: EnermyColor, 
    pub pos: Vec2, 
    charge_timer_optional: Option<f32>, // // used for mini enemies, that home in on player
}


pub enum EnermyState {
    Homing(EnermyStateHoming), 
    Normal(EnermyStateNormal), 
    Spawning(EnermyStateSpawning), 
    Shooting(EnermyStateShooting)
}


#[derive(PartialEq)]
pub struct EnermyStateNormal {
    shoot_timer: f32
}


#[derive(PartialEq)]
pub struct EnermyStateShooting {
    shots_left: i32,
    shoot_timer: f32,
}


#[derive(PartialEq)]
pub struct EnermyStateHoming {}


#[derive(PartialEq)]
pub struct EnermyStateSpawning {
    spawn_timer: f32,
}



pub struct Enermy {
    pub state_shared: EnermyStateShared, 
    pub state:EnermyState
}

#[derive(Clone, Copy)]
pub enum EnermyDeathMethod {
    None, 
    SpawnChildren(i32)
}





impl Enermy {
    pub fn new(
        pos: Vec2, 
        texture: Texture2D, 
        health: i32, 
        death_method: EnermyDeathMethod, 
        enermy_type: EnermyType, 
        enermy_color: EnermyColor
    ) -> Self {
        let charge_timer_optional = match enermy_type {
            EnermyType::NORMAL => None, 
            EnermyType::MINI => Some(rand::gen_range(
                ENERMY_MINI_HOMING_TIME_RANGE.x,
                ENERMY_MINI_HOMING_TIME_RANGE.y
            )),
        }; 

        Enermy {
            state_shared: EnermyStateShared {
                pos, 
                texture, 
                health, 
                angle: 0f32, 
                death_method, 
                animation_timer: 0f32,
                enermy_color, 
                enermy_type,
                angle_speed: rand::gen_range(100 as f32, 50 as f32), 
                collision_rect: Rect::new(0f32, 0f32, texture.width(), texture.height()), 
                charge_timer_optional,
            }, 
            state: EnermyState::Spawning(EnermyStateSpawning { spawn_timer: 0f32})
        }

    }

    pub fn update(
        &mut self, 
        dt: f32, 
        bullets: &mut Vec<Bullet>, 
        resources: &Resources, 
        player_pos: &Vec2, 
        game_manager: &mut WaveManager, 
        sound_mixer: &mut SoundMixer
    ) {

        let command_optional = match &mut self.state {
            EnermyState::Spawning(state_data) => {
                Self::update_state_spawning(&mut self.state_shared, dt, state_data)
            }
            EnermyState::Normal(state_data) => {
                Self::update_state_normal(&mut self.state_shared, dt, state_data)
            }

            EnermyState::Shooting(state_data) => Self::update_state_shooting(
                &mut self.state_shared,
                dt, 
                bullets,
                resources, 
                state_data, 
                sound_mixer
            ),

            EnermyState::Homing(state_data) => Self::update_state_homing(
                &mut self.state_shared,
                dt,
                player_pos,
                game_manager,
                sound_mixer,
                resources,
            )
        };

        match command_optional {
            None => {}
            Some(command) => match command {
                EnermyCommand::ChangeState(new_state) => {
                    self.state = new_state; 
                }
            }
        }
    }

    pub fn draw_state_spawning_normal(state_shared: &EnermyStateShared, state_data: &EnermyStateSpawning) {
        let rand_frame = rand::gen_range(0i32, 2i32);
        let fraction = 1.0f32 - state_data.spawn_timer / ENERMY_MINI_ANIM_TIME_SPAWN; 
        let offset = fraction * ENERMY_MINI_ANIM_TIME_SPAWN; 
        let spirit_width = state_shared.texture.width() /3f32; 
        let scale = spirit_width + fraction * ENERMY_MINI_ANIM_TIME_SPAWN * spirit_width; 

        //left-wing
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x - ((state_shared.texture.width() / 3.0f32) * 1.0f32) - offset,
            state_shared.pos.y,
            WHITE, 
            DrawTextureParams {
                rotation: 0f32, 
                dest_size: Some(vec2(scale, state_shared.texture.height())), 
                source: Some(Rect::new(
                    state_shared.texture.width() / 3f32 * rand_frame as f32, 
                    0f32, 
                    state_shared.texture.width() /3f32, 
                    state_shared.texture.height()
                )),
                ..Default::default()
            },
            
        );

        //right-wing
        draw_texture_ex(
            state_shared.texture,
            state_shared.pos.x + offset,
            state_shared.pos.y,
            WHITE,
            DrawTextureParams {
                rotation: 0f32,
                flip_x: true,
                dest_size: Some(vec2(scale, state_shared.texture.height())),
                source: Some(Rect::new(
                    state_shared.texture.width() / 3f32 * rand_frame as f32,
                    0f32,
                    state_shared.texture.width() / 3f32,
                    state_shared.texture.height(),
                )),
                ..Default::default()
            },
        );
    }

    pub fn draw_state_spawning_mini (state_shared: &EnermyStateShared, state_data: &EnermyStateSpawning) {
        let rand_frame = rand::gen_range(0i32, 2i32);
        let fraction = state_data.spawn_timer / ENERMY_MINI_ANIM_TIME_SPAWN;
        let spirit_width = state_shared.texture.width() / 4f32; 
        let scale = spirit_width * 0.5f32 + fraction * 1.5f32 * spirit_width; 

        draw_texture_ex(
            state_shared.texture, 
            state_shared.pos.x - ((state_shared.texture.width() / 4.0f32) * 1.0f32), 
            state_shared.pos.y,
            WHITE, 
            DrawTextureParams {
                rotation: fraction * std::f32::consts::PI, 
                dest_size: Some(vec2(scale, scale)), 
                source: Some(Rect::new(
                    state_shared.texture.width() / 4f32 * rand_frame as f32,
                    0f32,
                    state_shared.texture.width() / 4f32,
                    state_shared.texture.height(),
                )),
                ..Default::default()
            }
        ); 

        //right-wing
        draw_texture_ex(
            state_shared.texture, 
            state_shared.pos.x, 
            state_shared.pos.y, 
            WHITE, 
            DrawTextureParams {
                rotation: fraction * std::f32::consts::PI * 2f32, 
                flip_x: true, 
                dest_size: Some(vec2(scale, scale)), 
                source: Some(Rect::new(
                    state_shared.texture.width() / 4f32 * rand_frame as f32, 
                    0f32, 
                    state_shared.texture.width() / 4f32, 
                    state_shared.texture.height(),
                )),
                ..Default::default()

            }

        )
    }

    pub fn draw_state_spawning(state_shared: &EnermyStateShared, state_data: &EnermyStateSpawning) {
        match state_shared.enermy_type {
            EnermyType::NORMAL => Self::draw_state_spawning_normal(state_shared, state_data),
            EnermyType::MINI => Self::draw_state_spawning_mini(state_shared, state_data)
        }
    }

    fn draw_state_normal(&self) {
        let rand_frame = (self.state_shared.animation_timer / ENERMY_ANIM_TIME_FLAP).floor(); 
        //left-wing
        draw_texture_ex(
            self.state_shared.texture,
            self.state_shared.pos.x, 
            self.state_shared.pos.y, 
            WHITE,
            DrawTextureParams {
                rotation: 0f32, 
                source: Some(Rect::new(
                    self.state_shared.texture.width() /4f32 * rand_frame as f32, 
                    0f32, 
                    self.state_shared.texture.width() / 4f32, 
                    self.state_shared.texture.height()
                )),
                ..Default::default()
            }
        ); 

        //right-wing 
        draw_texture_ex(
            self.state_shared.texture,
            self.state_shared.pos.x, 
            self.state_shared.pos.y, 
            WHITE, 
            DrawTextureParams {
                rotation: 0f32, 
                flip_x: true, 
                source: Some(Rect::new(
                    self.state_shared.texture.width() / 4f32, 
                    0f32, 
                    self.state_shared.texture.width() / 4f32, 
                    self.state_shared.texture.height()
                )), 
                ..Default::default()
            }
        )
    }

    pub fn overlaps(&self, other_rect: &Rect) -> bool {
        self.state_shared.collision_rect.overlaps(other_rect)
    }

    pub fn clamp_in_view(pos: &mut Vec2) {
        let x_padding = 4f32;
        if pos.x < x_padding {
            pos.x = x_padding;
        } else if pos.x > GAME_SIZE_X as f32 - x_padding {
            pos.x = GAME_SIZE_X as f32 - x_padding
        }
        
        let top_padding = 7f32;
        let bottom_padding = 60f32;
        if pos.y < top_padding {
            pos.y = top_padding;
        } else if pos.y > GAME_SIZE_Y as f32 - bottom_padding {
            pos.y = GAME_SIZE_Y as f32 - bottom_padding;
        }
    }

    fn update_state_spawning(
        state_shared: &mut EnermyStateShared,
        dt: f32,
        state_data: &mut EnermyStateSpawning,
    )-> Option<EnermyCommand>{
        state_data.spawn_timer += dt;

        let end_time = match state_shared.enermy_type {
            EnermyType::MINI => ENERMY_MINI_ANIM_TIME_SPAWN, 
            EnermyType::NORMAL => ENEMY_ANIM_TIME_SPAWN
        }; 

        let fraction = state_data.spawn_timer / end_time; 
        if fraction >= 1.0f32 {
            return Some(EnermyCommand::ChangeState(EnermyState::Normal(EnermyStateNormal { shoot_timer : 0f32 })))
        }

        None
    }

    fn update_state_normal(
        state_shared: &mut EnermyStateShared,
        dt: f32,
        state_data: &mut EnermyStateNormal,
    ) -> Option<EnermyCommand>{
        let angle_change_speed = std::f32::consts::PI * state_shared.angle_speed; 
        state_shared.angle += (get_time() as f32 * angle_change_speed).sin() * std::f32::consts::PI * 2f32 * dt;
        let dir = vec2(state_shared.angle.sin(), -state_shared.angle.cos());
        state_shared.pos.x += dt * ENERMY_SPEED * dt; 


        
        Self::clamp_in_view(&mut state_shared.pos); 
        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width() * 0.5f32; 
        state_shared.collision_rect.y = state_shared.pos.y;
        state_data.shoot_timer += dt; 


        if let Some(charge_timer) = &mut state_shared.charge_timer_optional {
            *charge_timer -= dt;
            if *charge_timer <= 0f32  {
                return Some(EnermyCommand::ChangeState(EnermyState::Homing(
                    EnermyStateHoming {}, 
                )))
            }
        }

        state_shared.animation_timer += dt; 

        if state_shared.animation_timer > ENERMY_ANIM_TIME_FLAP * 4f32 {
            state_shared.animation_timer -= ENERMY_ANIM_TIME_FLAP * 4f32; 
        }


        if state_data.shoot_timer > ENERMY_SHOOT_TIME {
            let shot_count = rand::gen_range(1, ENERMY_MAX_BURST_COUNT); 
            
        }

        None
    }

    fn update_state_shooting(
        state_shared: &mut EnermyStateShared,
        dt: f32,
        bullets: &mut Vec<Bullet>,
        resources: &Resources,
        state_data: &mut EnermyStateShooting,
        sound_mixer: &mut SoundMixer,
    ) -> Option<EnermyCommand> {

        state_shared.pos.x += rand::gen_range(-1f32, 1f32) * ENERMY_SPEED * 0.5f32 * dt;
        state_shared.pos.y += rand::gen_range(-1f32, 1f32) * ENERMY_SPEED * 0.5f32 * dt;
        Self::clamp_in_view(&mut state_shared.pos); 
        state_data.shoot_timer -= dt;

        if state_data.shoot_timer <= 0f32 {
            state_data.shoot_timer = ENERMY_SHOOT_BURST_TIME; 
            state_data.shots_left -= 1;

            let should_spawn_2 = rand::gen_range(0, 2) > 1;
            if should_spawn_2 {
                let spawn_offset = vec2((state_shared.texture.width() / 4f32) * 0.5f32, 0f32); 
                bullets.push(Bullet::new(state_shared.pos - spawn_offset, BulletHurtType::Player, resources))
            }else {
                let spawn_offset = vec2(0f32, -3f32);
                bullets.push(Bullet::new(state_shared.pos + spawn_offset, BulletHurtType::Player, resources))
            }

            resources.play_sound(SoundIdentifier::EnermyShoot, sound_mixer, Volume(1.0f32)); 
            state_shared.pos.y -= 2f32;
        }


        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width() * 0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y; 

        state_shared.animation_timer += dt;
        if state_shared.animation_timer > ENERMY_ANIM_TIME_FLAP * 4f32 {
            state_shared.animation_timer -= ENERMY_ANIM_TIME_FLAP * 4f32; 
        }

        if state_data.shots_left <= 0 { //dont shootin the player
            return Some(EnermyCommand::ChangeState(EnermyState::Normal(
                EnermyStateNormal { shoot_timer: 0f32 },
            )))
        }
        None
    }

    fn update_state_homing (
        state_shared: &mut EnermyStateShared,
        dt: f32,
        player_pos: &Vec2,
        game_manager: &mut WaveManager,
        sound_mixer: &mut SoundMixer,
        resources: &Resources,
    ) -> Option<EnermyCommand> {
        state_shared.animation_timer += dt;
        if state_shared.animation_timer >= ENERMY_ANIM_TIME_FLAP * 4f32 {
            state_shared.animation_timer -= ENERMY_ANIM_TIME_FLAP  * 4f32; 
            resources.play_sound(SoundIdentifier::Warning, sound_mixer, Volume(1.0f32))
        }

        let player_dx = player_pos.x - state_shared.pos.x; 
        let sway_speed = 20f32; 
        let dx = if player_dx > 0f32 { 1f32 } else {-1f32 };
        let sway = (get_time() as f32 * sway_speed).sin(); 
        let sway = (sway + 1f32 ) * 0.5f32; 

        let vel = vec2(dx * ENERMY_SPEED_HOMING.x * sway, ENERMY_SPEED_HOMING.y);
        state_shared.pos += vel * dt;

        state_shared.collision_rect.x = state_shared.pos.x - state_shared.texture.width() * 0.5f32;
        state_shared.collision_rect.y = state_shared.pos.y;

        if state_shared.pos.y > GAME_SIZE_X  as f32  {
            state_shared.health = 0; 
            game_manager.last_enermydeath_reason = LastEnermyDeathReason::Environment; 
        }
        
        None
    }


    pub fn draw(&mut self) {
        match &self.state {
            EnermyState::Spawning(state_data) => {
                Self::draw_state_spawning(&self.state_shared, state_data)
            }

            EnermyState::Normal(state_data) => self.draw_state_normal(),
            EnermyState::Shooting(state_data) => self.draw_state_normal(),
            EnermyState::Homing(state_data) => self.draw_state_normal()
        }
    }
}