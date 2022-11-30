use bevy::{
    prelude::{Entity, Resource},
    utils::Instant,
};

use self::{fleet::Fleet, galaxy::Star};

pub mod fleet;
mod galaxy;
pub mod setup;
pub mod starfield;
pub mod turns;
pub mod ui;
pub mod world;

#[derive(Clone, Copy)]
struct StarDetails {
    population: f32,
    resources: f32,
    owner: usize,
    owned_since: u32,
}

#[derive(Resource)]
pub struct Universe {
    galaxy: Vec<Star>,
    players: Vec<Player>,
    star_entities: Vec<Entity>,
    star_details: Vec<StarDetails>,
}

#[derive(Resource)]
pub struct FleetsToSpawn(pub Vec<Fleet>);

impl Universe {
    fn star_revenue(&self, star_index: usize) -> f32 {
        let details = self.star_details[star_index];
        let good_conditions = &self.galaxy[self.players[details.owner].start];
        let star = &self.galaxy[star_index];
        if star.color == good_conditions.color {
            (details.population * 1.1).powf(1.4) / 100.0 - 2.0
        } else {
            (details.population).powf(0.8) / 100.0 - 2.0
        }
    }

    fn star_ressource(&self, star_index: usize) -> f32 {
        let details = self.star_details[star_index];
        let good_conditions = &self.galaxy[self.players[details.owner].start];
        let star = &self.galaxy[star_index];
        if star.color != good_conditions.color {
            (details.resources * 1.2).powf(1.5)
        } else {
            (details.resources).powf(0.8)
        }
    }

    fn player_population(&self, player: usize) -> f32 {
        self.star_details
            .iter()
            .filter(|details| details.owner == player)
            .map(|details| details.population)
            .sum()
    }

    fn player_revenue(&self, player: usize) -> f32 {
        let good_conditions = &self.galaxy[self.players[player].start];

        self.star_details
            .iter()
            .zip(self.galaxy.iter())
            .filter(|(details, _)| details.owner == player)
            .map(|(details, star)| {
                if star.color == good_conditions.color {
                    (details.population * 1.1).powf(1.4) / 100.0 - 2.0
                } else {
                    (details.population).powf(0.8) / 100.0 - 2.0
                }
            })
            .sum::<f32>()
    }
}

struct Player {
    start: usize,
    vision: Vec<StarState>,
    savings: f32,
    resources: f32,
    first_colony_done: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum StarState {
    Owned(usize),
    Unknown,
    Uninhabited,
}

#[derive(Resource)]
pub struct CurrentGame {
    start: Instant,
}

mod z_levels {
    pub const STARFIELD: f32 = 0.0;
    pub const STAR_SELECTION: f32 = 0.4;
    pub const STAR: f32 = 0.5;
    pub const STAR_DECORATION: f32 = 0.6;
    pub const STAR_NAME: f32 = 0.7;
    pub const SHIP: f32 = 0.8;
    pub const SHIP_DRAGGING: f32 = 1.0;
}

pub const PLAYER_NAMES: [&str; 4] = ["Violetta", "Papageno", "Figaro", "Gilda"];
