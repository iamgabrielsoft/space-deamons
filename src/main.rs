
use macroquad::prelude::*;
use quad_snd::mixer::SoundMixer; 
use game::{ GameManager, GameStateMenu, GameStateIdentifier, GameStateGame, GameState};


use resources::load_resouces; 
use constants::*; 
use enermy::*; 
use player::*; 
use wave::*; 
use bullet::*; 



mod resources; 
mod constants;
mod enermy;
mod game;
mod player; 
mod wave; 
mod bullet; 


fn window_conf() -> Conf {
    Conf {
        window_title : "SPACE_DEAMONS".to_owned(), 
        window_width : GAME_SIZE_X,  
        window_height : GAME_SIZE_Y, 
        ..Default::default()
    }
}


pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    return std::mem::discriminant(a) == std::mem::discriminant(b); 
}


#[macroquad::main(window_conf())]
async fn main() {

    let game_renderer = render_target(GAME_SIZE_X as u32, GAME_SIZE_Y as u32);
    let resources = load_resouces(game_renderer).await; 
    let mixer = SoundMixer::new(); 
    let game_states: Vec<(GameStateIdentifier, Box<dyn GameState>)> = vec![
        (GameStateIdentifier::Menu, Box::new(GameStateMenu::new())),
        (
            GameStateIdentifier::Game,
            Box::new(GameStateGame::new(&resources)),
        ),
    ];


    let mut gameManager = GameManager::new(game_states, resources, mixer); 
  
    loop {
        let dt = get_frame_time(); 
        let camera = Camera2D {
            zoom: vec2(1. / GAME_SIZE_X as f32 * 2., 1. / GAME_SIZE_Y as f32 * 2.),
            target: vec2(
                (GAME_SIZE_X as f32 * 0.5f32).floor(), 
                (GAME_SIZE_Y as f32 * 0.5f32).floor()
                ), 
                render_target: Some(game_renderer), 
                ..Default::default()
        }; 
        set_camera(&camera); 
        clear_background(BLACK); 

        gameManager.update(dt);
        gameManager.draw(); 

        set_default_camera(); 

        let game_diff_w = screen_width() / GAME_SIZE_X as f32; 
        let game_diff_h = screen_height() / GAME_CENTER_Y as f32; 
        let aspect_diff = game_diff_w.min(game_diff_h); 

        let scaled_game_size_w = GAME_SIZE_X as f32 * aspect_diff;
        let scaled_game_size_h = GAME_SIZE_Y as f32 * aspect_diff;

            let width_padding = (screen_width()- scaled_game_size_w) * 0.5f32; 
        let height_padding = (screen_height() - scaled_game_size_h) * 0.5f32;

        draw_texture_ex(
            game_renderer.texture, 
            width_padding,
            height_padding,
            WHITE, 
            DrawTextureParams {
                dest_size: Some(Vec2::new(scaled_game_size_w, scaled_game_size_h)),
                ..Default::default()
            }

        ); 


        gameManager.draw_unscaled(); 
        gameManager.frame_sound(); 

        next_frame().await
    }

  
}
