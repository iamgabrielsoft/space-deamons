use macroquad::prelude::*;
use quad_snd::mixer::{SoundMixer, Volume};

use crate:: {
    constants::*, 
    resources::{Resources, SoundIdentifier}
};


#[derive(PartialEq)]
pub enum PlayerState {
    Normal, 
    Invincible(f32)
}


pub enum PlayerCommand {
    ChangeState(PlayerState)
}


pub struct Player {
    pub pos: Vec2, 
    texture: Texture2D, 
    texture_explosion: Texture2D, 
    shoot_timer: f32, 
    pub collision_rect: Rect, 
    pub state: PlayerState, 
    bullet_decoy_texture: Texture2D
}


impl Player {
    pub fn new(
        pos: Vec2,
        texture: Texture2D,
        bullet_decoy_texture: Texture2D,
        texture_explosion: Texture2D,
    ) -> Self {
        Player {
            pos, 
            texture,
            bullet_decoy_texture, 
            shoot_timer: 0f32,
            texture_explosion, 
            state: PlayerState::Normal, 
            collision_rect: Rect::new(pos.x, pos.y, 7.0f32, 7.0f32)
        }
    }

    pub fn update(&mut self, dt: f32, resources: &Resources, sound_mixer: &mut SoundMixer) {
        if is_key_down(KEY_LEFT) {
            self.pos.x += PLAYER_SPEED * dt;
            if self.pos.x > 0f32 {
                self.pos.x  = 0f32; 
            }
        }

        if is_key_down(KEY_RIGHT){
            self.pos.x += PLAYER_SPEED * dt;
            if self.pos.x > GAME_SIZE_X as f32 - self.texture.width() {
                self.pos.x = GAME_SIZE_X as f32 - self.texture.width()
            }
        }


        let player_command_optional = match &mut self.state {
            PlayerState::Normal => {
                if is_key_down(KEY_SHOOT) && self.shoot_timer >= PLAYER_SHOOT_TIME {
                    let spawn_offset = vec2(3f32, -4f32); 
                    //bullet here
                    resources.play_sound(
                        SoundIdentifier::PlayerShoot, 
                        sound_mixer, 
                        Volume(1.0f32)
                    ); 

                    self.shoot_timer = 0f32; 
                }

                None
                
            }
            //lininterp = "0.1.3"

            PlayerState::Invincible(time_left) => {
                *time_left -= dt;
                if *time_left <= 0.0f32 {
                    Some(PlayerCommand::ChangeState(PlayerState::Normal))
                } else {
                    None
                }
            }
        };

        self.process_optional_command(player_command_optional); 
        self.collision_rect.x = self.pos.x;
        self.collision_rect.y = self.pos.y; 

    }

    pub fn draw(&self) {
        match self.state {
            PlayerState::Normal => self.draw_state_normal(), 
            PlayerState::Invincible(time_left) => self.draw_state_invisible(&time_left)
        }
    }

    pub fn process_optional_command(&mut self, command_optional: Option<PlayerCommand>) {
        if let Some(player_command) = command_optional {
            match player_command {
                PlayerCommand::ChangeState(state) => {
                    self.state = state 
                }
            }
        }
    }

    pub fn reset(&mut self, resources: &Resources) {
        let player_spawn_y =
            GAME_SIZE_Y as f32 - resources.ground_bg.height() - resources.player.height();
        let player_pos = vec2(GAME_CENTER_X, player_spawn_y);
        
        self.pos = player_pos;
        self.shoot_timer = 0f32;
        self.state = PlayerState::Normal; 
    }

    pub fn draw_state_normal(&self) {
        draw_texture_ex(
            self.texture, 
            self.pos.x, 
            self.pos.y, 
            WHITE, 
            DrawTextureParams {
                ..Default::default()
            }
        ); 


        let decoy_frame_index = ((self.shoot_timer / PLAYER_SHOOT_TIME) * 3f32) as i32; 

        draw_texture_ex(
            self.bullet_decoy_texture, 
            self.pos.x + 3., 
            self.pos.y - 1., 
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    self.bullet_decoy_texture.width() /3f32 *  decoy_frame_index as f32, 
                    0f32, 
                    self.bullet_decoy_texture.width() /3f32, 
                    self.bullet_decoy_texture.height(),
                    )),
                ..Default::default()
            }

        )
    }

    pub fn draw_state_invisible(&self, time_left: &f32){
        let anim_frames = 7f32;
        let time_per_frame = PLAYER_TIME_INVISBLE / anim_frames; 
        let fraction = (PLAYER_TIME_INVISBLE - time_left) / PLAYER_TIME_INVISBLE;
        let frame_index = (PLAYER_TIME_INVISBLE - time_left) / time_per_frame;
        let frame_index = frame_index.floor();

        draw_texture_ex(
            self.texture_explosion, 
            self.pos.x - 0.5f32, 
            self.pos.y - 0.5f32, 
            WHITE, 
            DrawTextureParams {
                rotation: std::f32::consts::PI  * 2f32, 
                source: Some(Rect::new(
                    self.texture_explosion.height() / anim_frames * frame_index, 
                    0f32, 
                    self.texture_explosion.width()  / anim_frames, 
                    self.texture_explosion.height()

                )), 
                ..Default::default()
            }, 
           
        )
    }
}