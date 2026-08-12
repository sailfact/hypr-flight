use bevy::prelude::*;

#[derive(Resource)]
pub struct Tuning {
    pub playfield: Vec2,
    pub ship_radius: f32,
    pub thrust: f32,
    pub turn_rate: f32,
    pub drag: f32,
    pub max_speed: f32,
    pub bullet_radius: f32,
    pub bullet_speed: f32,
    pub bullet_lifetime: f32,
    pub fire_cooldown: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            playfield: Vec2::new(1280.0, 720.0),
            ship_radius: 12.0,
            thrust: 380.0,
            turn_rate: 3.6,
            drag: 0.6,
            max_speed: 520.0,
            bullet_radius: 2.5,
            bullet_speed: 620.0,
            bullet_lifetime: 1.1,
            fire_cooldown: 0.09,
        }
    }
}
