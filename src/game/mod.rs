use bevy::prelude::Resource;

use self::galaxy::Star;

mod galaxy;
pub mod setup;
pub mod world;

#[derive(Resource)]
struct World {
    galaxy: Vec<Star>,
    start: Vec<usize>,
}
