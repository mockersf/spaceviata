use std::f32::consts::PI;

use bevy::prelude::{Component, Resource, Vec2};
use rand::{distributions::WeightedIndex, prelude::Distribution, seq::SliceRandom, Rng};

#[derive(Clone, Copy, Debug, Default)]
pub enum GalaxyKind {
    #[default]
    Spiral,
}

#[derive(Resource)]
pub struct GalaxyCreator {
    pub nb_players: u32,
    pub size: f32,
    pub density: f32,
    pub _kind: GalaxyKind,
    pub generated: Vec<Vec2>,
}

pub struct Star {
    pub position: Vec2,
    pub size: StarSize,
    pub color: StarColor,
}

#[derive(Component, Copy, Clone)]
pub enum StarColor {
    Blue,
    Yellow,
    Orange,
}

#[derive(Component, Copy, Clone)]
pub enum StarSize {
    Dwarf,
    Subgiant,
    Giant,
}

impl From<StarSize> for f32 {
    fn from(size: StarSize) -> Self {
        match size {
            StarSize::Dwarf => 0.75,
            StarSize::Subgiant => 1.25,
            StarSize::Giant => 3.0,
        }
    }
}

impl Iterator for GalaxyCreator {
    type Item = Star;

    fn next(&mut self) -> Option<Self::Item> {
        if self.generated.len() as f32 == self.nb_players as f32 * self.size * self.density * 4.0 {
            return None;
        }

        let mut rand = rand::thread_rng();
        let arm_angle = ((360 / self.nb_players) % 360) as f32;
        let angular_spread = 180 / (self.nb_players * 2);

        let mut fail = 0;

        let size_choices = [StarSize::Dwarf, StarSize::Subgiant, StarSize::Giant];
        let size_weights = [30, 30, 1];
        let size_dist = WeightedIndex::new(&size_weights).unwrap();

        'distance: loop {
            let distance_to_center =
                rand.gen_range(0.03..=1.0_f32).sqrt() * self.size as f32 * 100.0;
            let angle = rand.gen_range(0.0..(angular_spread as f32));

            let spiral_angle = 0.75;

            let arm = (rand.gen::<u32>() % self.nb_players) as f32 * arm_angle;

            let x = distance_to_center
                * (PI / 180.0 * (arm + distance_to_center * spiral_angle + angle) as f32).cos();
            let y = distance_to_center
                * (PI / 180.0 * (arm + distance_to_center * spiral_angle + angle) as f32).sin();
            let new_star = Vec2::new(x, y);

            for other_star in &self.generated {
                let distance = new_star.distance(*other_star);
                if distance < 100.0 / (self.density as f32) {
                    fail += 1;
                    if distance < 100.0 / (self.density as f32 * (1.0 + fail as f32 / 1000.0)) {
                        continue 'distance;
                    }
                }
            }
            self.generated.push(new_star);
            return Some(Star {
                position: new_star,
                size: size_choices[size_dist.sample(&mut rand)],
                color: *[StarColor::Blue, StarColor::Orange, StarColor::Yellow]
                    .choose(&mut rand)
                    .unwrap(),
            });
        }
    }
}
