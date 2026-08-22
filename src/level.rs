use bevy::prelude::*;

use crate::shapes::ShapeAssets;

/// Spec section 11. `Level` owns this; `Tuning` must not duplicate it.
pub const TILE_SIZE: f32 = 128.0;

/// Where the ship spawns, and respawns after death (spec section 10).
pub const START_TILE: IVec2 = IVec2::new(7, 7);

// pub struct LevelDef {
//     pub name: &'static str,
//     pub ascii: &'static str,
// }

pub const LEVEL: &str = include_str!("../levels/space_test");

/// Marker for a rendered solid tile.
#[derive(Component)]
pub struct Wall;

/// The tile grid. World origin is the grid's bottom-left corner, so tile
/// `(x, y)` occupies `[x * tile_size, (x+1) * tile_size)` on X and the
/// equivalent on Y.
#[derive(Resource)]
pub struct Level {
    solid: Vec<bool>,
    width: usize,
    height: usize,
    tile_size: f32,
}

impl Level {
    /// Parse a level from ASCII. `#` is solid, `.` is open.
    ///
    /// The string is written top-down for readability: the FIRST line is the
    /// highest y, the LAST line is y = 0. Blank lines are ignored so the const
    /// can be indented in source.
    pub fn from_ascii(src: &str, tile_size: f32) -> Self {
        let rows: Vec<&str> = src
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect();

        assert!(!rows.is_empty(), "level is empty");
        let height = rows.len();
        let width = rows[0].len();
        assert!(
            rows.iter().all(|row| row.len() == width),
            "ragged level rows: expected every row to be {width} wide"
        );

        let mut solid = vec![false; width * height];
        for (row_index, row) in rows.iter().enumerate() {
            let y = height - 1 - row_index;
            for (x, ch) in row.chars().enumerate() {
                solid[y * width + x] = ch == '#';
            }
        }

        Self {
            solid,
            width,
            height,
            tile_size,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_size(&self) -> f32 {
        self.tile_size
    }

    /// Anything outside the grid counts as solid, so callers never need a
    /// separate bounds check.
    pub fn is_solid(&self, tile: IVec2) -> bool {
        if tile.x < 0 || tile.y < 0 {
            return true;
        }
        let (x, y) = (tile.x as usize, tile.y as usize);
        if x >= self.width || y >= self.height {
            return true;
        }
        self.solid[y * self.width + x]
    }

    /// Which tile contains this world point. Uses `floor`, so negative
    /// coordinates map outside the grid rather than folding onto tile 0.
    pub fn world_to_tile(&self, point: Vec2) -> IVec2 {
        IVec2::new(
            (point.x / self.tile_size).floor() as i32,
            (point.y / self.tile_size).floor() as i32,
        )
    }

    /// World-space centre of a tile. Used for spawning and rendering.
    pub fn tile_center(&self, tile: IVec2) -> Vec2 {
        Vec2::new(
            (tile.x as f32 + 0.5) * self.tile_size,
            (tile.y as f32 + 0.5) * self.tile_size,
        )
    }

    /// True if nothing solid lies between the two world points.
    ///
    /// Amanatides-Woo grid traversal: step to whichever axis boundary is
    /// nearer, test that tile, repeat until the accumulated distance exceeds
    /// the segment length. Endpoints inside a solid tile count as blocked.
    #[allow(dead_code)]
    pub fn line_of_sight(&self, from: Vec2, to: Vec2) -> bool {
        let mut tile = self.world_to_tile(from);
        if self.is_solid(tile) {
            return false;
        }

        let delta = to - from;
        let distance = delta.length();
        if distance <= f32::EPSILON {
            return true;
        }
        let dir = delta / distance;

        let step = IVec2::new(
            if dir.x >= 0.0 { 1 } else { -1 },
            if dir.y >= 0.0 { 1 } else { -1 },
        );

        // How far along the ray one full tile of travel costs, per axis.
        let t_delta = Vec2::new(
            if dir.x != 0.0 {
                (self.tile_size / dir.x).abs()
            } else {
                f32::INFINITY
            },
            if dir.y != 0.0 {
                (self.tile_size / dir.y).abs()
            } else {
                f32::INFINITY
            },
        );

        // Distance along the ray to the first boundary crossing, per axis.
        let next_boundary = Vec2::new(
            (tile.x + if step.x > 0 { 1 } else { 0 }) as f32 * self.tile_size,
            (tile.y + if step.y > 0 { 1 } else { 0 }) as f32 * self.tile_size,
        );
        let mut t_max = Vec2::new(
            if dir.x != 0.0 {
                (next_boundary.x - from.x) / dir.x
            } else {
                f32::INFINITY
            },
            if dir.y != 0.0 {
                (next_boundary.y - from.y) / dir.y
            } else {
                f32::INFINITY
            },
        );

        while t_max.x.min(t_max.y) < distance {
            if t_max.x < t_max.y {
                tile.x += step.x;
                t_max.x += t_delta.x;
            } else {
                tile.y += step.y;
                t_max.y += t_delta.y;
            }
            if self.is_solid(tile) {
                return false;
            }
        }

        true
    }

    /// World-space spawn point for the player.
    pub fn start_position(&self) -> Vec2 {
        self.tile_center(START_TILE)
    }
}

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, build_level)
            .add_systems(Startup, spawn_wall_tiles);
    }
}

/// Spec section 7.1: built at `PreStartup` so every `Startup` system can read it.
fn build_level(mut commands: Commands) {
    commands.insert_resource(Level::from_ascii(LEVEL, TILE_SIZE));
}

/// One quad entity per solid tile. A few thousand entities that batch to a
/// single draw call — adequate at prototype scale (spec section 7.1).
fn spawn_wall_tiles(mut commands: Commands, level: Res<Level>, shapes: Res<ShapeAssets>) {
    for y in 0..level.height() as i32 {
        for x in 0..level.width() as i32 {
            let tile = IVec2::new(x, y);
            if !level.is_solid(tile) {
                continue;
            }
            let centre = level.tile_center(tile);
            commands.spawn((
                Wall,
                Mesh2d(shapes.tile_mesh.clone()),
                MeshMaterial2d(shapes.tile_material.clone()),
                Transform::from_xyz(centre.x, centre.y, -1.0),
            ));
        }
    }
}
