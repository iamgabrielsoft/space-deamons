
use macroquad::prelude::*;

use crate::{ resources::Resources, constants::* }; 

#[derive(std::cmp::PartialEq)]
pub enum BulletHurtType {
    Player, 
    Enermy
}

pub struct Bullet {
    texture: Texture2D, 
    pos: Vec2, 
    vel: Vec2, 
    pub hurt_type: BulletHurtType, 
    anim_timer: f32,
    pub collision_rect: Rect, 
    pub is_kill: bool 
}



impl Bullet {

    pub fn new (pos: Vec2, hurt_type: BulletHurtType, resources: Resources) -> Self {
        let (vel, texture) = match hurt_type {
            BulletHurtType::Enermy => (
                vec2(0f32, -1f32 * PLAYER_BULLET_SPEED),
                resources.player_missle
            ), 
            BulletHurtType::Player => (vec2(0f32, ENERMY_BULLET_SPEED), resources.deamon_missle)
           
        }; 

        Bullet {
            pos, 
            texture, 
            vel, 
            hurt_type, 
            anim_timer: 0f32, 
            collision_rect: Rect::new(pos.x, pos.y, 2.0f32, 6f32), 
            is_kill: false
        }
    }

    pub fn draw(&mut self) {
        let frame = ((self.anim_timer / BULLET_ANIM_TIME_SPAWN * 3.0f32)) as i32; 
        draw_texture_ex(
            self.texture, 
            self.pos.x, 
            self.pos.y, 
            WHITE, 
            DrawTextureParams {
                rotation: 0f32, 
                source: Some(Rect::new(
                    self.texture.width() /3f32  * frame as f32, 
                    0f32, 
                    self.texture.width() / 3f32, 
                    self.texture.height()
                )),
                ..Default::default()
            }
        )
    }

    pub fn overlaps(&self, other_rect: &Rect) -> bool {
        return self.collision_rect.overlaps(other_rect); 
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt; 
        self.anim_timer += dt; 
        self.collision_rect.x = self.pos.x; 
        self.collision_rect.y = self.pos.y; 

    }


}