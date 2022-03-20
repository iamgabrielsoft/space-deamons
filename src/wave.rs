
use macroquad::prelude::*; 
use quad_snd::mixer::{ SoundMixer, Volume}; 

use crate::{
    constants::*, 
    enermy::{ EnermyColor, EnermyType, Enermy, EnermyDeathMethod},
    resources::{ Resources, SoundIdentifier }
}; 


pub struct WaveManagerStateSpawning {
    enermies_left: i32,
    spawn_timer: f32
}



pub enum WaveManagerState {
    Spawning(WaveManagerStateSpawning),
    Battle
}


// used to internally modify gamestate
pub enum WaveManagerCommand {
    ChangeState(WaveManagerState),
}



// used to get information from gamestate
pub enum WaveManagerMessage {
    LevelCleared,
}

#[derive(PartialEq)]
pub enum LastEnermyDeathReason {
    Environment, 
    Player
}



pub struct WaveManager {
    pub state: WaveManagerState,
    pub last_enermydeath_reason: LastEnermyDeathReason, 
    internal_timer: f32
}


impl WaveManager {
    pub fn new () -> Self {
        let enermies_left = ENERMY_SPAWN_STARTING_COUNT; 
        
        WaveManager {
            state: WaveManagerState::Spawning(WaveManagerStateSpawning {
                spawn_timer: 0f32, 
                enermies_left,
            }),
            last_enermydeath_reason: LastEnermyDeathReason::Environment,
            internal_timer: 0f32
        }
    }

    fn get_enermy_spawn_count(time: &f32) -> i32 {
        let fraction = time / TIME_UNTIL_MAX_DIFFICULTY;
        let spawn_count = lininterp::lerp(
            &(ENERMY_SPAWN_STARTING_COUNT as f32), 
            &(ENERMY_SPAWN_MAX_COUNT as f32),
            &fraction,
        ); 
        
        return spawn_count as i32; 
    }

    pub fn update(
        &mut self, 
        dt: f32, 
        enermies: &mut Vec<Enermy>,
        resources: &Resources, 
        sound_mixer: &mut SoundMixer
     ) -> Option<WaveManagerMessage> {
        self.internal_timer += dt;
        let state_command_optional = match &mut self.state {
            WaveManagerState::Spawning(game_state_spawing) => Self::update_state_spawning(
                game_state_spawing, 
                dt, 
                enermies, 
                resources, 
                sound_mixer
            ), 
            WaveManagerState::Battle => Self::update_state_battle(enermies, &self.internal_timer)
        };


        if let Some(state_command) = state_command_optional {
            match state_command {
                WaveManagerCommand::ChangeState(target_state) => {
                    self.state = target_state;

                    // let cleared_screen = variant_eq {
                    //     &self.state

                    // }; 


                    // if cleared_screen {
                    //     return Some(WaveManagerMessage::LevelCleared); 
                    // }
                }
            }
        }
        None
    }

    pub fn update_state_spawning (
        game_state_spawning: &mut WaveManagerStateSpawning,
        dt: f32,
        enermies: &mut Vec<Enermy>, 
        resources: &Resources, 
        sound_mixer: &mut SoundMixer 
    ) -> Option<WaveManagerCommand> {
        game_state_spawning.spawn_timer += dt; 
        if game_state_spawning.spawn_timer > ENERMY_SPAWN_TIME {
            game_state_spawning.enermies_left -= 1; 
            game_state_spawning.spawn_timer -= ENERMY_SPAWN_TIME; 

            spawn_enermy(
                enermies,
                resources, 
                SpawnBlueprint::Normal, 
                EnermyColor::random()
            ); 
            resources.play_sound(SoundIdentifier::Spawn, sound_mixer, Volume(0.4f32)); 
        }

        if game_state_spawning.enermies_left <= 0 {
            return Some(WaveManagerCommand::ChangeState(WaveManagerState::Battle))
        }
        None
    }

    fn update_state_battle(
        enermies: &mut Vec<Enermy>, 
        internal_timer: &f32,
    ) -> Option<WaveManagerCommand> {
        if enermies.is_empty() {
            let enermies_left = Self::get_enermy_spawn_count(internal_timer); 

            return Some(WaveManagerCommand::ChangeState(WaveManagerState::Spawning(
                WaveManagerStateSpawning {
                    enermies_left, 
                    spawn_timer: 0f32
                }
            )))
        }
        None
    }
}


pub enum SpawnBlueprint {
    Normal, 
    Mini(Vec2)
}


pub fn spawn_enermy(
    enermies: &mut Vec<Enermy>, 
    resources: &Resources, 
    spawn_blueprint: SpawnBlueprint, 
    enermy_color: EnermyColor
) {
    let health = 1;
    let enermy = match spawn_blueprint {
        SpawnBlueprint::Normal => {
            let spawn_offset = vec2(
                rand::gen_range(-100f32, 100f32), 
                rand::gen_range(-60f32, 10f32)
            ); 

            let spawn_pos = vec2(GAME_CENTER_X, GAME_CENTER_Y) + spawn_offset; 
            let death_method = if rand::gen_range(0f32, 1f32) > 0.5f32 {
                let spawn_amount = rand::gen_range(1, 2 + 1); 
                EnermyDeathMethod::SpawnChildren(spawn_amount) 
            }else {
                EnermyDeathMethod::None
            };

            Enermy::new(
                spawn_pos, 
                resources.rand_enemy_normal(enermy_color), 
                health, 
                death_method, 
                EnermyType::NORMAL,
                enermy_color
            )
        }

        SpawnBlueprint::Mini(pos) => Enermy::new(
            pos, 
            resources.rand_enermy_mini(enermy_color),
            health, 
            EnermyDeathMethod::None, 
            EnermyType::MINI, 
            enermy_color
        ),
    };

    enermies.push(enermy);  
}