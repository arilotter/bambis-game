use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TileKind {
    Empty,
    Wall,
    Floor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tile {
    pub kind: TileKind,
}
