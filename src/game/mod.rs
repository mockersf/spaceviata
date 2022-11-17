use bevy::{
    prelude::{Entity, Resource},
    utils::Instant,
};

use self::galaxy::Star;

mod galaxy;
pub mod setup;
pub mod starfield;
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
}

mod z_levels {
    pub const STAR: f32 = 0.5;
    pub const STAR_NAME: f32 = 0.7;
    pub const STAR_DECORATION: f32 = 0.6;
    pub const STARFIELD: f32 = 0.0;
}
