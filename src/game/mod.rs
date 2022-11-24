use bevy::{
    prelude::{Entity, Resource},
    utils::Instant,
};

use self::galaxy::Star;

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
    owned_since: usize,
}

#[derive(Resource)]
struct Universe {
    galaxy: Vec<Star>,
    players: Vec<Player>,
    star_entities: Vec<Entity>,
    star_details: Vec<StarDetails>,
}

impl Universe {
    fn star_revenue(&self, star_index: usize) -> f32 {
        let details = self.star_details[star_index];
        let good_conditions = &self.galaxy[self.players[details.owner].start];
        let star = &self.galaxy[star_index];
        if star.color == good_conditions.color {
            (details.population * 1.2).powf(1.5) / 100.0
        } else {
            (details.population).powf(0.8) / 100.0
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
                    (details.population * 1.2).powf(1.5)
                } else {
                    (details.population).powf(0.8)
                }
            })
            .sum::<f32>()
            / 100.0
    }
}

struct Player {
    start: usize,
    vision: Vec<StarState>,
    savings: f32,
    resources: f32,
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
    pub const STAR_SELECTION: f32 = 0.4;
    pub const STAR_DECORATION: f32 = 0.6;
    pub const STARFIELD: f32 = 0.0;
}
