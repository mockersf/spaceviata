use bevy::prelude::*;
use rand::{seq::IteratorRandom, Rng};

use crate::game::fleet::{Fleet, FleetSize, ShipKind};

use super::{
    fleet::{turns_between, Order, Owner, Ship},
    galaxy::Star,
    turns::{TurnState, Turns},
    FleetsToSpawn, StarState, Universe,
};

#[derive(Resource, Default)]
pub struct BotTurnStatus {
    pub current: usize,
    pub last_colony_ship_spawned: Vec<u32>,
}

pub fn start_bots(mut bot_turn_status: ResMut<BotTurnStatus>) {
    bot_turn_status.current = 1;
}

fn rate_star_colony(
    player: usize,
    good: &Star,
    from: Vec2,
    state: &StarState,
    rating: &Star,
) -> u32 {
    turns_between(from, rating.position)
        + match state {
            StarState::Owned(i) if *i == player => 500,
            StarState::Owned(_) => 500000,
            StarState::Unknown => 100,
            StarState::Uninhabited => {
                (good.color != rating.color)
                    .then_some(15)
                    .unwrap_or_default()
                    + (good.size != rating.size).then_some(15).unwrap_or_default()
            }
        }
}

fn rate_star_fighter(
    player: usize,
    from: Vec2,
    fleet_size: &FleetSize,
    state: &StarState,
    rating: &Star,
) -> u32 {
    turns_between(from, rating.position)
        + match state {
            StarState::Owned(i) if *i == player => 500,
            StarState::Owned(_) => {
                if fleet_size.0 < 10 {
                    500000
                } else {
                    50
                }
            }
            StarState::Unknown => 50,
            StarState::Uninhabited => 100,
        }
}

pub fn run_bots_turn(
    mut status: ResMut<BotTurnStatus>,
    mut universe: ResMut<Universe>,
    mut state: ResMut<State<TurnState>>,
    mut fleets_to_spawn: ResMut<FleetsToSpawn>,
    mut fleets: Query<(&Ship, &mut Order, &Owner, &FleetSize)>,
    turns: Res<Turns>,
) {
    let current_bot = status.current;
    let starting_star = &universe.galaxy[universe.players[current_bot].start];

    for (ship, mut order, owner, fleet_size) in &mut fleets {
        if owner.0 != current_bot {
            continue;
        }
        if let Order::Orbit(n) = *order {
            let current_position = universe.galaxy[n].position;
            match ship.kind {
                ShipKind::Colony => {
                    let mut rated_stars = universe
                        .galaxy
                        .iter()
                        .zip(universe.players[current_bot].vision.iter())
                        .enumerate()
                        .filter(|(i, _)| *i != n)
                        .map(|(index, (star, state))| {
                            (
                                index,
                                rate_star_colony(
                                    current_bot,
                                    starting_star,
                                    current_position,
                                    state,
                                    star,
                                ),
                            )
                        })
                        .collect::<Vec<_>>();
                    rated_stars.sort_by_key(|i| i.1);
                    *order = Order::Move {
                        from: n,
                        to: rated_stars[0].0,
                        step: 0,
                    };
                }
                ShipKind::Fighter => {
                    let mut rated_stars = universe
                        .galaxy
                        .iter()
                        .zip(universe.players[current_bot].vision.iter())
                        .enumerate()
                        .filter(|(i, _)| *i != n)
                        .map(|(index, (star, state))| {
                            (
                                index,
                                rate_star_fighter(
                                    current_bot,
                                    current_position,
                                    fleet_size,
                                    state,
                                    star,
                                ),
                            )
                        })
                        .collect::<Vec<_>>();
                    rated_stars.sort_by_key(|i| i.1);
                    *order = Order::Move {
                        from: n,
                        to: rated_stars[0].0,
                        step: 0,
                    };
                }
            }
        }
    }

    let mut rand = rand::thread_rng();
    // enough revenue to create a new colony
    // enough credits & resources to build a colony ship
    // didn't create one very recently
    // there is an not owned star available
    if universe.player_revenue(current_bot) > 2.0
        && universe.players[current_bot].savings > ShipKind::Colony.cost_credits()
        && universe.players[current_bot].resources > ShipKind::Colony.cost_credits()
        && turns.count - status.last_colony_ship_spawned[current_bot] > 2
        && universe.players[current_bot]
            .vision
            .iter()
            .find(|state| !matches!(state, StarState::Owned(_)))
            .is_some()
    {
        let star = universe
            .galaxy
            .iter()
            .zip(universe.star_details.iter())
            .enumerate()
            .filter(|(_, (_, details))| details.owner == current_bot)
            .choose(&mut rand)
            .unwrap()
            .0;
        fleets_to_spawn.0.push(Fleet {
            order: Order::Orbit(star),
            ship: Ship {
                kind: ShipKind::Colony,
            },
            size: FleetSize(1),
            owner: Owner(current_bot),
        });
        universe.players[current_bot].savings -= ShipKind::Colony.cost_credits();
        universe.players[current_bot].resources -= ShipKind::Colony.cost_resources();
        status.last_colony_ship_spawned[current_bot] = turns.count;
    }

    let nb_fighter = rand.gen_range(1..(((turns.count as f32).ln() * 10.0) as u32 + 2));
    if universe.players[current_bot].savings > nb_fighter as f32 * ShipKind::Fighter.cost_credits()
        && universe.players[current_bot].resources
            > nb_fighter as f32 * ShipKind::Fighter.cost_resources()
    {
        let star = universe
            .galaxy
            .iter()
            .zip(universe.star_details.iter())
            .enumerate()
            .filter(|(_, (_, details))| details.owner == current_bot)
            .choose(&mut rand)
            .unwrap()
            .0;
        fleets_to_spawn.0.push(Fleet {
            order: Order::Orbit(star),
            ship: Ship {
                kind: ShipKind::Fighter,
            },
            size: FleetSize(nb_fighter),
            owner: Owner(current_bot),
        });
        universe.players[current_bot].savings -=
            ShipKind::Fighter.cost_credits() * nb_fighter as f32;
        universe.players[current_bot].resources -=
            ShipKind::Fighter.cost_resources() * nb_fighter as f32;
    }

    status.current += 1;
    if status.current == universe.players.len() {
        let _ = state.set(TurnState::Enemy);
    }
}
