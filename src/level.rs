use bevy::prelude::*;

/// `Level` owns this; `Tuning` must not duplicate it.
pub const TILE_SIZE: f32 = 32.0;

/// Where the ship spawns, and where it respawns after death
pub const START_TILE: IVec2 = IVec2::new(7, 7);

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
    /// highest y, and the LAST line is y = 0. Blank lines are ignored so the
    /// const can be indented in source.
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
    /// Anything outside the grid counts as solid, so callers never need a
    /// separate bounds check.
    pub fn is_solid(&self, tile: IVec2) -> bool {
        let (x, y) = (tile.x as usize, tile.y as usize);
        if tile.x < 0 || tile.y < 0 {
            return true;
        }
        if x >= self.width || y >= self.height {
            return true;
        }
        self.solid[y * self.width + x]
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

    pub fn world_to_tile(&self, point: Vec2) -> IVec2 {
        IVec2::new(
            (point.x / self.tile_size).floor() as i32,
            (point.y / self.tile_size).floor() as i32,
        )
    }

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
    fn build(&self, app: &mut App) {}
}
