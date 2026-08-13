pub mod game;
pub mod menu;
pub mod splash;

use bevy::prelude::*;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DisplayQuality {
    Low,
    Medium,
    High,
}

#[derive(Component)]
pub struct Setting<T>(T);

#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Volume(u32);
