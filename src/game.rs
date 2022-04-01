use macroquad::prelude::*; 
use std::collections::HashMap; 
use quad_snd::mixer::{ SoundMixer, Volume };

use crate::{
    constants::*,
    resources::{Resources, SoundIdentifier}, 
    player::{Player, PlayerState, PlayerCommand}, 
    enermy::{Enermy, EnermyColor, EnermyType, EnermyState, EnermyStateHoming, EnermyDeathMethod}, 
    variant_eq,
    wave::{
        WaveManagerState, LastEnermyDeathReason, WaveManager, WaveManagerMessage, spawn_enermy, SpawnBlueprint
    }, 
    bullet::{Bullet, BulletHurtType},
    
};





pub struct MenuPayload {
    score: i32,
}



pub struct GameStateMenu {
    last_score_optional: Option<i32>
}

pub enum ChangeStatePayload {
    MenuPayload(MenuPayload),
}

pub enum GameStateCommand {
    ChangeState(GameStateIdentifier, Option<ChangeStatePayload>),
}


impl GameStateMenu {
    pub fn new() -> Self {
        GameStateMenu {
            last_score_optional: None
        }
    }
}


pub fn draw_lives(player_lives: &i32, texture_life: Texture2D, texture_ground_bg: &Texture2D, game_manager: &WaveManager){
    let lives_padding = 2f32; 
    let wave_speed = 20f32;
    let last_kill_from_player = game_manager.last_enermydeath_reason == LastEnermyDeathReason::Player;
    let wave_offset_y = -7f32;
    let wave_time_offset = 0.7f32;
    match &game_manager.state {
        WaveManagerState::Spawning(_spawning_state) if last_kill_from_player => {
            for i in 0..*player_lives {
                let wave = ((get_time() as f32 * wave_speed + i as f32 * wave_time_offset).sin()
                + 1f32)
                * 0.5f32;

                draw_texture_ex(
                    texture_life,
                    5f32 + i as f32 * (texture_life.width() + lives_padding),
                    GAME_SIZE_Y as f32 - texture_ground_bg.height()  + 3f32 + wave + wave_offset_y,
                    PINK,
                    DrawTextureParams {
                        ..Default::default()
                    }
                )
            }

        } _ => {

            for i in 0..*player_lives {
                draw_texture_ex(
                    texture_life,
                    5f32 + i as f32 * (texture_life.width() + lives_padding),
                    GAME_SIZE_Y as f32 - texture_ground_bg.height() + 3f32, 
                    WHITE,
                    DrawTextureParams {
                        ..Default::default()
                    }
                )
            }
        }
    }
}

impl GameState for GameStateMenu {
    fn draw(&self, resources: &Resources) {
        draw_texture_ex(
            resources.ground_bg, 
            0f32,
            GAME_SIZE_X as f32  - resources.ground_bg.height(), 
            WHITE, 
            DrawTextureParams {
                dest_size: Some(Vec2::new(GAME_SIZE_X as f32, resources.ground_bg.height())),
                ..Default::default()
            }
        )
    }

    fn update(&mut self, dt: f32, resources: &Resources, sound_mixer: &mut SoundMixer) -> Option<GameStateCommand>{
        if is_key_pressed(KEY_START_GAME){
            return Some(GameStateCommand::ChangeState(GameStateIdentifier::Game, None,))
        }

        None
    }

    fn on_enter(&mut self, resources: &Resources, payload_optional: Option<ChangeStatePayload>){
        if let Some(payload) = payload_optional {
            match payload {
                ChangeStatePayload::MenuPayload(menu_payload) => {
                    self.last_score_optional = Some(menu_payload.score)
                }
            }
        }
    }

    fn draw_unscaled(&self, resources: &Resources){
        let game_diff_w = screen_width() / GAME_SIZE_X as f32;
        let game_diff_h = screen_height() / GAME_SIZE_Y as f32;
        let aspect_diff = game_diff_w.min(game_diff_h);

        let scaled_game_size_w = GAME_SIZE_X as f32 * aspect_diff;
        let scaled_game_size_h = GAME_SIZE_Y as f32 * aspect_diff;

        let width_padding = (screen_width() - scaled_game_size_w) * 0.5f32;
        let height_padding = (screen_height() - scaled_game_size_h) * 0.5f32;

        let font_size = (aspect_diff * 10f32) as u16;

        if let Some(last_score) = self.last_score_optional {
            let score_text = format!("{}", last_score);
            let mut text_x = width_padding + scaled_game_size_w * 0.5f32;
            text_x -= score_text.len() as f32 * 0.5f32 * font_size as f32 * 0.6f32;
            draw_text_ex(
                score_text.as_ref(),
                text_x,
                height_padding + font_size as f32 * 2f32,
                TextParams {
                    font: resources.font,
                    font_size,
                    font_scale: 1f32,
                    color: YELLOW,
                    font_scale_aspect: 1f32,
                },
            );
        }
        let start_text = "TAP SPACE TO START";
        let mut text_x = width_padding + scaled_game_size_w * 0.5f32;
        text_x -= start_text.len() as f32 * 0.5f32 * font_size as f32 * 0.6f32;

        draw_text_ex(
            start_text,
            text_x,
            screen_height() * 0.5f32,
            TextParams {
                font: resources.font,
                font_size,
                font_scale: 1f32,
                color: YELLOW,
                font_scale_aspect: 1f32,
            },
        );
    
    }
    
}

pub trait GameState {
    fn update(
        &mut self, 
        dt: f32, 
        resources: &Resources, 
        sound_mixer: &mut SoundMixer, 
    ) -> Option<GameStateCommand>; 
    fn draw(&self, resources: &Resources); 
    fn draw_unscaled(&self, resources: &Resources); 
    fn on_enter(&mut self, resources: &Resources, payload_optional: Option<ChangeStatePayload>);
}


#[derive(PartialEq, Eq, Hash)]
pub enum GameStateIdentifier {
    Menu, 
    Game
}


pub struct GameStateGame {
    player_score: i32, 
    player_lives: i32, 
    player: Player, 
    enermies: Vec<Enermy>, 
    bullet: Vec<Bullet>,
    wave_manager: WaveManager
}



//implementaation 
impl GameStateGame {
    pub fn new(resources: &Resources) -> Self {
        let player_spawn_y = GAME_SIZE_X as f32 - resources.ground_bg.height() - resources.ground_bg.width(); 
        let player_pos = vec2(GAME_CENTER_X, player_spawn_y); 

        let player = Player::new(
            player_pos, 
            resources.player, 
            resources.player_missle, 
            resources.player_explosion
        );

        GameStateGame {
            player_score: 0, 
            player_lives: PLAYER_LIVES_START, 
            player, 
            bullet: Vec::<Bullet>::new(), 
            enermies: Vec::<Enermy>::new(), 
            wave_manager: WaveManager::new()
        }

    }
}


impl GameState for GameStateGame {
    fn draw(&self, _resources: &Resources) {}
    fn draw_unscaled(&self, resources: &Resources) {
        let game_diff_w = screen_width() / GAME_SIZE_X as f32; 
        let game_diff_h = screen_height() / GAME_SIZE_Y as f32; 
        let aspect_diff = game_diff_w.min(game_diff_h); 


        let scaled_game_size_w = GAME_SIZE_X as f32 * aspect_diff; 
        let scaled_game_size_h = GAME_SIZE_Y as f32 * aspect_diff;

        let width_padding = (screen_width() - scaled_game_size_w) * 0.5f32;
        let height_padding = (screen_height() - scaled_game_size_h) * 0.5f32;


        let score_text = format!("{}", self.player_score); 
        let font_size = (aspect_diff * 10f32) as u16;
        let mut text_x = width_padding + scaled_game_size_w * 0.5f32;
        text_x -= score_text.len() as f32 * 0.5f32 * font_size as f32 * 0.6f32;
        draw_text_ex(
            score_text.as_ref(), 
            text_x,
            height_padding + font_size as f32 * 2f32, 
            TextParams {
                font: resources.font, 
                font_size,
                font_scale: 1f32,
                color: YELLOW,
                font_scale_aspect: 1f32
            }
        )


    }

    fn update(&mut self, dt: f32, resources: &Resources, sound_mixer: &mut SoundMixer) -> Option<GameStateCommand>{
        let manager_message_optional =
        self.wave_manager
            .update(dt, &mut self.enermies, resources, sound_mixer);
    if let Some(manager_message) = manager_message_optional {
        match manager_message {
            WaveManagerMessage::LevelCleared => {
                self.player_lives += 1;
                self.player_lives = self.player_lives.min(PLAYER_LIVES_MAX);
                let score_add = match self.wave_manager.last_enermydeath_reason {
                    LastEnermyDeathReason::Environment => SCORE_SURVIVED_ALL,
                    LastEnermyDeathReason::Player => SCORE_KILL_ALL,
                };
                resources.play_sound(SoundIdentifier::WaveCleared, sound_mixer, Volume(0.6f32));
                self.player_score += score_add;
            }
        }
    }

    for enemy in self.enermies.iter_mut() {
        enemy.update(
            dt,
            &mut self.bullet,
            resources,
            &self.player.pos,
            &mut self.wave_manager,
            sound_mixer,
        );
        enemy.draw();
    }

    for bullet in self.bullet.iter_mut() {
        bullet.update(dt);
        bullet.draw();
    }

    // bullets hurting player
    for bullet in self.bullet.iter_mut().filter(|b| b.hurt_type == BulletHurtType::Player){
        if bullet.overlaps(&self.player.collision_rect) {
            if self.player.state != PlayerState::Normal {
                continue;
            }
            self.player_lives -= 1;
            resources.play_sound(SoundIdentifier::PlayerOuch, sound_mixer, Volume(1.0f32));
            // CHANGE PLAYER STATE
            self.player.process_optional_command(Some(PlayerCommand::ChangeState(PlayerState::Invincible(PLAYER_TIME_INVISBLE),)));
            if self.player_lives <= 0 {
                return Some(GameStateCommand::ChangeState(
                    GameStateIdentifier::Menu,
                    Some(ChangeStatePayload::MenuPayload(MenuPayload {
                        score: self.player_score,
                    })),
                ));
            }
            bullet.is_kill = true;
            break;
        }
    }

    // homing enemies hurting player
    for enemy in self.enermies.iter_mut().filter(|e| variant_eq(&e.state, &EnermyState::Homing(EnermyStateHoming {})))
        // filter enemies containing homing state, variant_eq is used so we can disregard homing data
        
    {
        if enemy.overlaps(&self.player.collision_rect) {
            let player_invisible =
                variant_eq(&self.player.state, &PlayerState::Invincible(0f32));
            if !player_invisible {
                self.player_lives -= 1;
                resources.play_sound(SoundIdentifier::PlayerOuch, sound_mixer, Volume(1.0f32));
                self.player
                    .process_optional_command(Some(PlayerCommand::ChangeState(
                        PlayerState::Invincible(PLAYER_TIME_INVISBLE),
                    )));
                enemy.state_shared.health = 0;
            }
        }
    }

    // todo explain
    let mut death_methods = Vec::<(Vec2, EnermyDeathMethod, EnermyType, EnermyColor)>::with_capacity(4);

    // bullets hurting enemies
    for bullet in self.bullet.iter_mut().filter(|b| b.hurt_type == BulletHurtType::Enermy){
        for enemy in self.enermies.iter_mut() {
            if enemy.overlaps(&bullet.collision_rect) && !bullet.is_kill {
                enemy.state_shared.health -= 1;
                self.wave_manager.last_enermydeath_reason = LastEnermyDeathReason::Player;
                // death
                if enemy.state_shared.health <= 0 {
                    resources.play_sound(
                        SoundIdentifier::EnermyOuch,
                        sound_mixer,
                        Volume(1.0f32),
                    );
                    death_methods.push((
                        enemy.state_shared.pos,
                        enemy.state_shared.death_method,
                        enemy.state_shared.enermy_type,
                        enemy.state_shared.enermy_color,
                    ));
                }
                // can only hurt one enemy, flag for deletion
                bullet.is_kill = true;
            }
        }
    }

    for (pos, death_method, enemy_type, enemy_color) in death_methods.iter() {
        let score_add = match enemy_type {
            EnermyType::NORMAL => SCORE_NORMAL,
            EnermyType::MINI => SCORE_MINI,
        };
        self.player_score += score_add;
        match death_method {
            EnermyDeathMethod::None => {}
            EnermyDeathMethod::SpawnChildren(amount) => {
                resources.play_sound(SoundIdentifier::SpawnMini, sound_mixer, Volume(1.0f32));
                let spawn_width = 20f32;
                let step = 1. / (*amount as f32);
                for i in 0..*amount {
                    let spawn_pos = *pos + vec2(step * spawn_width * i as f32, 0f32);
                    spawn_enermy(
                        &mut self.enermies,
                        resources,
                        SpawnBlueprint::Mini(spawn_pos),
                        *enemy_color,
                    );
                }
            }
        }
    }

    
    self.bullet.retain(|e| !e.is_kill);// remove bullets that hit something
    self.enermies.retain(|e| e.state_shared.health > 0); // remove dead enemies

    draw_texture_ex(
        resources.ground_bg,
        0f32,
        GAME_SIZE_Y as f32 - resources.ground_bg.height(),
        WHITE,
        DrawTextureParams {
            //dest_size: Some(vec2(screen_width(), screen_height())),
            dest_size: Some(Vec2::new(GAME_SIZE_X as f32, resources.ground_bg.height())),
            ..Default::default()
        },
    );

        draw_lives(
            &self.player_lives,
            resources.life,
            &resources.ground_bg,
            &self.wave_manager,
        );

        self.player.update(dt, &mut self.bullet, resources, sound_mixer);
        self.player.draw();
        None
           
    }

    fn on_enter(&mut self, resources: &Resources, payload_optional: Option<ChangeStatePayload>) {
        self.wave_manager.reset(); 
        self.player.reset(resources); 
        self.player_score = 0; 
        self.player_lives = PLAYER_LIVES_START; 
        self.enermies.clear(); 
        self.bullet.clear(); 
    }
}


pub struct GameManager {
    states: HashMap<GameStateIdentifier, Box<dyn GameState>>,
    current_state_identifier: GameStateIdentifier,
    resources: Resources, 
    sound_mixer: SoundMixer
}



impl GameManager {
    pub fn new(
        all_states: Vec<(GameStateIdentifier, Box<dyn GameState>)>, 
        resources:Resources, 
        sound_mixer: SoundMixer
    ) -> Self {
        let mut states = HashMap::new(); 
        for state in all_states.into_iter() {
            states.insert(state.0, state.1); 
        }

        GameManager {
            states, 
            current_state_identifier: GameStateIdentifier::Menu,
            resources, 
            sound_mixer
        }
    }

    pub fn frame_sound(&mut self) {
        self.sound_mixer.frame(); 
    }


    pub fn update(&mut self, dt: f32){
        let state_command_optional = 
            if let Some(game_state) = self.states.get_mut(&self.current_state_identifier){
                game_state.update(dt, &self.resources, &mut self.sound_mixer) 
            }else {
                None
            };

            if let Some(state_command) = state_command_optional {
                    match state_command {
                        GameStateCommand::ChangeState(next_state, payload_optional) => {
                            self.current_state_identifier = next_state;
                            if let Some(game_state) = self.states.get_mut(&self.current_state_identifier) {
                                game_state.on_enter(&self.resources, payload_optional)
                            }
                        }
                    }
            }

    }

    pub fn draw(&self) {
        if let Some(game_state) = self.states.get(&self.current_state_identifier) {
            game_state.draw(&self.resources); 
        }
    }

    pub fn draw_unscaled(&self) {
        if let Some(game_state) = self.states.get(&self.current_state_identifier) {
            game_state.draw_unscaled(&self.resources); // the scaled proportion 
        }
    }
}