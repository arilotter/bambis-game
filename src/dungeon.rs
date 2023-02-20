use rand::{rngs::SmallRng, SeedableRng};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum TileKind {
    Empty,
    Wall,
    Floor,
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub kind: TileKind,
}

pub struct Dungeon {
    tiles: Vec<Vec<Tile>>,
    spawn_points: Vec<(usize, usize)>,
    spawn_index: usize,
}

impl Dungeon {
    pub fn gen(seed: u64) -> Dungeon {
        let mut rng = SmallRng::seed_from_u64(seed);
        let dungeon_size = 40.0;
        let max_room_size = dungeon_size / 2.0;
        let num_rooms = (dungeon_size * 2.0) as usize;
        let rooms: Vec<_> = (0..num_rooms)
            .map(|_| {
                let size = Vec2::new(
                    rng.gen_range(1.0..max_room_size),
                    rng.gen_range(1.0..max_room_size),
                )
                .round();

                let pos = random_point_in_circle(&mut rng, dungeon_size).round();
                Room {
                    x: pos.x,
                    y: pos.y,
                    width: size.x,
                    height: size.y,
                }
            })
            .collect();

        let left = rooms
            .iter()
            .map(|r| r.x)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();
        let right = rooms
            .iter()
            .map(|r| r.x + r.width + 1.0)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();

        let top = rooms
            .iter()
            .map(|r| r.y)
            .min_by(|a, b| a.total_cmp(b))
            .unwrap();
        let bottom = rooms
            .iter()
            .map(|r| r.y + r.height + 1.0)
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        for room in rooms.iter() {
            if !room.is_connected_to_another(&rooms) {
                println!("All rooms should be connected");
            }
        }

        let mut tiles: Vec<Vec<Tile>> = iter_float(top..=bottom, 1.0)
            .map(|y| {
                iter_float(left..=right, 1.0)
                    .map(|x| {
                        let point = Vec2::new(x, y);
                        let mut kind = TileKind::Empty;
                        for r in rooms.iter() {
                            if r.point_is_inside(point) {
                                if x >= right || x <= left || y >= bottom || y <= top {
                                    kind = TileKind::Wall;
                                } else {
                                    kind = TileKind::Floor;
                                }
                                break;
                            }
                            if r.point_is_on_wall(point) {
                                kind = TileKind::Wall;
                            }
                        }

                        Tile { kind }
                    })
                    .collect()
            })
            .collect();
        tiles.truncate(5);
        let mut spawn_points = Vec::with_capacity(4);
        'outer: for (y, row) in tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                if spawn_points.len() >= 4 {
                    break 'outer;
                }
                if matches!(tile.kind, TileKind::Floor) {
                    spawn_points.push((x, y))
                }
            }
        }
        Dungeon {
            tiles,
            spawn_points,
            spawn_index: 0,
        }
    }
    pub fn get_spawn_point(&mut self) -> (usize, usize) {
        let point = self.spawn_points[self.spawn_index];
        self.spawn_index += 1;
        self.spawn_index %= self.spawn_points.len();
        point
    }
    pub fn get_tiles(&self) -> Vec<Vec<Tile>> {
        self.tiles.clone()
    }
}

pub struct Room {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Room {
    fn point_is_inside(&self, point: Vec2) -> bool {
        point.x > self.x
            && point.x < self.x + self.width
            && point.y > self.y
            && point.y < (self.y + self.height)
    }
    fn point_is_on_wall(&self, point: Vec2) -> bool {
        ((point.x == self.x || point.x == self.x + self.width)
            && point.y >= self.y
            && point.y <= (self.y + self.height))
            || ((point.y == self.y || point.y == self.y + self.height)
                && point.x >= self.x
                && point.x <= (self.x + self.width))
    }

    fn is_connected_to_another(&self, all_rooms: &[Room]) -> bool {
        for r in all_rooms {
            for y in iter_float(self.y..=self.y + self.height, 1.0) {
                // left
                if r.point_is_inside(Vec2::new(self.x - 1.0, y)) {
                    return true;
                }
                // right
                if r.point_is_inside(Vec2::new(self.x + self.width + 1.0, y)) {
                    return true;
                }
            }
            for x in iter_float(self.x..=self.x + self.width, 1.0) {
                // top
                if r.point_is_inside(Vec2::new(x, self.y - 1.0)) {
                    return true;
                }
                // bottom
                if r.point_is_inside(Vec2::new(x, self.y + self.height + 1.0)) {
                    return true;
                }
            }
        }
        return false;
    }
}
