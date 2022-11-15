use bevy::{
    prelude::{Entity, Resource},
    utils::Instant,
};

use self::galaxy::Star;

mod galaxy;
pub mod setup;
pub mod world;

#[derive(Resource)]
struct World {
    galaxy: Vec<Star>,
    players: Vec<Player>,
    star_entities: Vec<Entity>,
}

struct Player {
    start: usize,
    vision: Vec<StarState>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum StarState {
    Owned(usize),
    Unknown,
}

#[derive(Resource)]
pub struct CurrentGame {
    start: Instant,
    init: bool,
}
