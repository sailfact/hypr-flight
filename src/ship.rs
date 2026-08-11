use bevy::prelude::*;

#[derive(Component)]
pub struct Ship {
    thrust: f32,
    turn_rate: f32,
    cooldown: Timer,
}
