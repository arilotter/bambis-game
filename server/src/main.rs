use bevy::prelude::*;
use bevy_slinet::{
    connection::ConnectionId,
    server::{NewConnectionEvent, PacketReceiveEvent, ServerConnections, ServerPlugin},
};
use dungeon::gen_dungeon;
use proto::{
    dungeon::{Tile, TileKind},
    *,
};

mod dungeon;
mod math;

#[derive(Component)]
struct Dungeon {
    tiles: Vec<Vec<Tile>>,
    spawn_points: Vec<(usize, usize)>,
    spawn_index: usize,
}
impl Dungeon {
    fn gen() -> Self {
        let tiles = gen_dungeon();
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
        Self {
            tiles,
            spawn_points,
            spawn_index: 0,
        }
    }
    fn get_spawn_point(&mut self) -> (usize, usize) {
        let point = self.spawn_points[self.spawn_index];
        self.spawn_index += 1;
        point
    }
    fn get_tiles(&self) -> Vec<Vec<Tile>> {
        self.tiles.clone()
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugin(ServerPlugin::<NetworkingConfig>::bind("0.0.0.0:3000"))
        .add_system(new_connection_system)
        .add_system(packet_receive_system)
        .add_startup_system(gen_dungeon_system)
        .run()
}

fn gen_dungeon_system(mut commands: Commands) {
    commands.spawn(Dungeon::gen());
}

fn new_connection_system(
    mut commands: Commands,
    mut events: EventReader<NewConnectionEvent<NetworkingConfig>>,
    characters: Query<&Character>,
    mut dungeon: Query<&mut Dungeon>,
) {
    // associate a player ID with the connection
    for event in events.iter() {
        let new_id = characters.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        event
            .connection
            .send(ServerPacket::IdentifyClient(new_id))
            .unwrap();
        let mut d = dungeon.single_mut();

        event
            .connection
            .send(ServerPacket::Dungeon {
                tiles: d.get_tiles(),
                spawn_point: d.get_spawn_point(),
            })
            .unwrap();
        commands
            .spawn(Character::new(new_id))
            .insert(event.connection.id());
    }
}

fn packet_receive_system(
    mut events: EventReader<PacketReceiveEvent<NetworkingConfig>>,
    connections: Res<ServerConnections<NetworkingConfig>>,
    characters: Query<(&Character, &ConnectionId)>,
) {
    for event in events.iter() {
        match &event.packet {
            // when we get a position from a player, rebroadcast it to other players.
            ClientPacket::Position(pos) => {
                let char = characters
                    .iter()
                    .find(|c| c.1 == &event.connection.id())
                    .unwrap();
                let packet = ServerPacket::PlayerPosition(char.0.id, *pos);
                for conn in connections.iter() {
                    if &conn.id() != char.1 {
                        conn.send(packet.clone()).unwrap();
                    }
                }
            }
        }
    }
}
