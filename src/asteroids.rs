use std::prelude::*;

#[derive(Component)]
pub struct Asteroid {
    pub size: AsteroidSize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AsteroidSize {
    Small,
    Medium,
    Large,
}

impl AsteroidSize {
    pub fn radius(self) -> f32 {
        match self {
            Self::Small => 6.0,
            Self::Medium => 11.0,
            Self::Large => 18.0,
        }
    }
}

#[derive(Resource)]
pub struct SpawnZones(Vec<IVec2>);

fn build_spawn_zones(mut commands: Commands, level: Res<Level>) {
    let mut zones = Vec::new();
    for y in 0..level.height() as i32 {
        for x in 0..level.width() as i32 {
            let clear =
                (-2..=2).all(|dy| (-2..=2).all(|dx| !level.is_solid(IVec::new(x + dx, y + dy))));
            if clear {
                zones.push(IVec2::new(x, y));
            }
        }
    }
    commands.insert_resource(SpawnZones(zones));
}
