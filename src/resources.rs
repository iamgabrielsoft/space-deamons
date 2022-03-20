use macroquad::prelude::*;
use quad_snd::{
    decoder::read_wav_ext, 
    mixer::{PlaybackStyle, SoundMixer}, 
    mixer::{ Sound, Volume}
}; 



use std::collections::HashMap; 
use crate::enermy::{EnermyColor, EnermyType}; 


pub struct Resources {
    pub life: Texture2D, 
    pub font: Font, 
    pub ground_bg: Texture2D, 
    pub player: Texture2D, 
    pub player_explosion: Texture2D, 
    pub player_missle: Texture2D, 
    pub deamon_missle: Texture2D, 


    pub demons_normal_purple: Vec<Texture2D>,
    pub demons_normal_green: Vec<Texture2D>,
    pub demons_normal_red: Vec<Texture2D>,
    pub demons_mini_purple: Vec<Texture2D>,
    pub demons_mini_green: Vec<Texture2D>,
    pub demons_mini_red: Vec<Texture2D>,

    pub sounds: HashMap<SoundIdentifier, Sound>
}


#[derive(PartialEq, Eq, Hash)]
pub enum SoundIdentifier {
    EnermyShoot,
    EnermyOuch,
    PlayerOuch,
    PlayerShoot,
    SpawnMini,
    Spawn,
    Warning,
    WaveCleared,
}


//let implement this resouces listed above
impl Resources {
    pub fn new(
        deamon_missle: Texture2D, 
        player: Texture2D, 
        player_explosion: Texture2D, 
        ground_bg: Texture2D, 
        life: Texture2D,
        player_missle: Texture2D,
        font: Font
    ) -> Self {
        Resources {
            demons_normal_green: Vec::<Texture2D>::new(), 
            demons_normal_purple: Vec::<Texture2D>::new(), 
            demons_normal_red: Vec::<Texture2D>::new(),
            demons_mini_green: Vec::<Texture2D>::new(), 
            demons_mini_purple: Vec::<Texture2D>::new(), 
            demons_mini_red: Vec::<Texture2D>::new(),
            deamon_missle, 
            font, 
            player, 
            ground_bg, 
            player_missle, 
            player_explosion,
            life, 
            sounds: HashMap::new()
        }
    }


    pub fn load_sound(&mut self, bytes: &[u8], identifier: SoundIdentifier)  {
        let sound = read_wav_ext(bytes, PlaybackStyle::Once).unwrap(); 
        self.sounds.insert(identifier, sound); 
    }

    pub fn play_sound(&self, identifier: SoundIdentifier, mixer: &mut SoundMixer, volume: Volume) {
        if let Some(sound) = self.sounds.get(&identifier) {
            mixer.play_ext(sound.clone(), volume); 
        }
    }


    pub async fn load_texture(
        &mut self,
        file_name: &str,
        enemy_color: EnermyColor,
        enemy_type: EnermyType,
    ) -> Result<(), FileError> {
        let texture: Texture2D = load_texture(file_name).await?;
        texture.set_filter(FilterMode::Nearest);
        let texture_vec = match enemy_type {
            EnermyType::NORMAL => match enemy_color {
                EnermyColor::PURPLE => &mut self.demons_normal_purple,
                EnermyColor::GREEN => &mut self.demons_normal_green,
                EnermyColor::RED => &mut self.demons_normal_red,
            },
            EnermyType::MINI => match enemy_color {
                EnermyColor::PURPLE => &mut self.demons_mini_purple,
                EnermyColor::GREEN => &mut self.demons_mini_green,
                EnermyColor::RED => &mut self.demons_mini_red,
            },
        };
        texture_vec.push(texture);
        Ok(())
    }


    pub fn rand_enemy_normal(&self, enermy_color: EnermyColor ) -> Texture2D{
        let normal_list = match enermy_color {
            EnermyColor::PURPLE => &self.demons_normal_purple, 
            EnermyColor::GREEN => &self.demons_normal_green, 
            EnermyColor::RED => &self.demons_normal_red
        }; 

        normal_list[rand::gen_range(0, normal_list.len())]
    }

    pub fn rand_enermy_mini(&self, enermy_color: EnermyColor) -> Texture2D {
        let mini_list = match enermy_color {
            EnermyColor::PURPLE => &self.demons_mini_purple, 
            EnermyColor::GREEN => &self.demons_mini_green, 
            EnermyColor::RED => &self.demons_mini_red
        }; 

        mini_list[rand::gen_range(0, mini_list.len())]
    }



    
}


const SOUND_BYTES_SPAWN: &[u8] = include_bytes!("../assets/sounds/spawn.wav");
const SOUND_BYTES_ENEMY_SHOOT: &[u8] = include_bytes!("../assets/sounds/enermy_shoot.wav");
const SOUND_BYTES_PLAYER_SHOOT: &[u8] = include_bytes!("../assets/sounds/player_shoot.wav");
const SOUND_BYTES_WAVE_CLEARED: &[u8] = include_bytes!("../assets/sounds/wave_cleared.wav");
const SOUND_BYTES_PLAYER_OUCH: &[u8] = include_bytes!("../assets/sounds/player_ouch.wav");


pub async fn load_resouces(game_render_target: RenderTarget ) -> Resources {
    
    let texture_player: Texture2D = load_texture("assets/player.png").await.unwrap();
    let texture_player_explosion: Texture2D = load_texture("assets/player_explotion.png").await.unwrap(); 
    let texture_player_missile: Texture2D = load_texture("assets/player_missile.png").await.unwrap(); 
    let texture_demon_missile: Texture2D = load_texture("assets/demon_missile.png").await.unwrap(); 
    let texture_ground_bg: Texture2D = load_texture("assets/ground_bg.png").await.unwrap();
    let texture_life: Texture2D = load_texture("assets/life.png").await.unwrap();
    

    for texture in [
        texture_player,
        texture_player_explosion,
        texture_player_missile,
        texture_demon_missile,
        texture_ground_bg,
        texture_life,
        game_render_target.texture,
    ].iter() {
        texture.set_filter(FilterMode::Nearest)
    }

    let font: Font = load_ttf_font("assets/Kenney Pixel Square.ttf").await.unwrap(); 
    let mut resources = Resources::new(
        texture_demon_missile, 
        texture_player_missile, 
        texture_player, 
        texture_ground_bg, 
        texture_life, 
        texture_player_missile, 
        font
    ); 

    {
        use EnermyColor::{GREEN, PURPLE, RED}; 
        use EnermyType:: { MINI, NORMAL }; 
        resources.load_texture("assets/demon_mini_green_1.png", GREEN, MINI).await.unwrap(); 
        resources.load_texture("assets/demon_mini_red_1.png", RED, MINI).await.unwrap();
        resources.load_texture("assets/demon_mini_purple_1.png", PURPLE, MINI).await.unwrap();
        resources.load_texture("assets/demon_normal_green_1.png", GREEN, NORMAL).await.unwrap();
        resources.load_texture("assets/demon_normal_green_2.png", GREEN, NORMAL).await.unwrap();
        resources.load_texture("assets/demon_normal_purple_1.png", PURPLE, NORMAL).await.unwrap();
        resources.load_texture("assets/demon_normal_purple_2.png", PURPLE, NORMAL).await.unwrap();
        resources.load_texture("assets/demon_normal_red_1.png", RED, NORMAL).await.unwrap();
    } 
    {
        use SoundIdentifier::*; 
        resources.load_sound(SOUND_BYTES_ENEMY_SHOOT, EnermyShoot); 
        resources.load_sound(SOUND_BYTES_PLAYER_SHOOT, PlayerShoot); 
    }

    resources
}