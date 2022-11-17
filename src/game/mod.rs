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
    init: bool,
}

mod z_levels {
    pub const STARS: f32 = 0.5;
    pub const STAR_NAMES: f32 = 0.6;
    pub const STARFIELD: f32 = 0.0;
}
