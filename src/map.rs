use rltk::*;
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use std::collections::HashSet;

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

/// Enum differentiating floor tiles from wall tiles.
#[derive(Eq, PartialEq, Copy, Clone, Hash, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

/// Structure for holding game map-related information.
///
/// `revealed_tiles`: `true` if the tile has been in our fov before, else `false`.
/// `visible_tiles`: `true` if the tile is currently in our fov, else `false`.
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    /// Generates a new, empty map.
    pub fn new(new_depth: i32) -> Map {
        Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth,
            bloodstains: HashSet::new(),
        }
    }

    /// Returns the (x, y) coordinates of the map's center.
    pub fn center(&self) -> (i32, i32) {
        (self.width / 2, self.height / 2)
    }

    /// Checks if movement by (d_x, d_y) amount will violate map bounds.
    pub fn in_bounds(&self, x: i32, d_x: i32, y: i32, d_y: i32) -> bool {
        x + d_x >= 1 && x + d_x < self.width - 1 && y + d_y >= 1 && y + d_y < self.height - 1
    }

    /// Gets 1D index (for [`Map::tiles`]) from passed 2D coordinates.
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    /// Determines if an index can be entered (is not blocked).
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    /// Sets all wall tiles to blocking tiles--can't walk through walls.
    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    /// Removes entities from all tiles.
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    // Iterates (x, y) coordinates in the map.
    pub fn iter_xy(&self) -> Vec<(i32, i32)> {
        (1..self.height - 1)
            .flat_map(|y| std::iter::repeat(y).zip(1..self.width - 1))
            .map(|(y, x)| (x, y))
            .collect::<Vec<(i32, i32)>>()
    }

    pub fn count_floor_tiles(&self) -> usize {
        self.tiles.iter().filter(|t| **t == TileType::Floor).count()
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    /// Returns `true` if a tile is a wall tile, else returns `false`.
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx as usize] == TileType::Wall
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.))
        };

        // Diagonal directions
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }
}

/// Renders the map to the terminal screen.
pub fn draw_map(map: &Map, ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;

    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending on its tile type.
        if map.revealed_tiles[idx] {
            // `glyph` and `fg` switches based on TileType.
            let glyph: FontCharType;
            let mut fg: RGB;
            let mut bg: RGB = RGB::from_f32(0., 0., 0.);
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = wall_glyph(&*map, x, y);
                    fg = RGB::from_f32(0., 1., 0.);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0., 1., 0.);
                }
            }
            // If tile isn't currently visible (but has been encountered),
            // render it in greyscale.
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale();
                bg = RGB::from_f32(0., 0., 0.);
            } else if map.bloodstains.contains(&idx) {
                // If this tile is bloodied, render it.
                bg = RGB::from_f32(0.75, 0., 0.);
            }
            ctx.set(x, y, fg, bg, glyph);
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
}

/// Applies bitmask to TileType.
fn wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    // Stay in the map bounds, please.
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 as i32 {
        return 35;
    }

    // 4-bit bitmask, since four directions a wall can be in.
    let mut mask: u8 = 0;
    if is_revealed_and_wall(map, x, y - 1) {
        mask += 1;
    }
    if is_revealed_and_wall(map, x, y + 1) {
        mask += 2;
    }
    if is_revealed_and_wall(map, x - 1, y) {
        mask += 4;
    }
    if is_revealed_and_wall(map, x + 1, y) {
        mask += 8;
    }

    match mask {
        0 => 9,    // Pillar (can't see neighbors)
        1 => 186,  // Wall to the north
        2 => 186,  // Wall to the south
        3 => 186,  // Wall to the north and south
        4 => 205,  // Wall to the west
        5 => 188,  // Wall to the north and west
        6 => 187,  // Wall to the south and west
        7 => 185,  // Wall to the north, south, and west
        8 => 205,  // Wall to the east
        9 => 200,  // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south, and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and south
        14 => 203, // Wall to the east, west, and north
        15 => 206, // Wall on all sides
        _ => 35,   // Just in case we missed one
    }
}

fn is_revealed_and_wall(map: &Map, x: i32, y: i32) -> bool {
    let idx = map.xy_idx(x, y);
    map.tiles[idx] == TileType::Wall && map.revealed_tiles[idx]
}
