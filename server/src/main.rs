use bevy::prelude::*;
use bevy_slinet::{
    connection::ConnectionId,
    server::{NewConnectionEvent, PacketReceiveEvent, ServerConnections, ServerPlugin},
};
use dungeon::gen_dungeon;
use proto::{dungeon::Tile, *};

mod dungeon;
mod math;

#[derive(Component)]
struct Dungeon(Vec<Vec<Tile>>);
impl Dungeon {
    fn new(dungeon: Vec<Vec<Tile>>) -> Self {
        Self(dungeon)
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
    commands.spawn(Dungeon::new(gen_dungeon()));
}

fn new_connection_system(
    mut commands: Commands,
    mut events: EventReader<NewConnectionEvent<NetworkingConfig>>,
    characters: Query<&Character>,
    dungeon: Query<&Dungeon>,
) {
    // associate a player ID with the connection
    for event in events.iter() {
        let new_id = characters.iter().map(|c| c.id).max().unwrap_or(0) + 1;
        event
            .connection
            .send(ServerPacket::IdentifyClient(new_id))
            .unwrap();
        eprintln!("sending dungeon");
        event
            .connection
            .send(ServerPacket::Dungeon(dungeon.single().0.clone()))
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
