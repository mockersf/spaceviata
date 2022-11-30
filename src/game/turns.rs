use bevy::{prelude::*, utils::HashMap};

use crate::assets::{GalaxyAssets, UiAssets};

use super::{
    fleet::{turns_between, FleetSize, Order, Owner, Ship, ShipKind},
    galaxy::StarColor,
    world::{StarHat, StarMask},
    StarState, Universe, PLAYER_NAMES,
};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum TurnState {
    Player,
    Bots,
    Enemy,
    Out,
}

#[derive(Resource)]
pub struct Turns {
    pub count: u32,
    pub messages: Vec<Message>,
}

pub enum Message {
    Turn(u32),
    ColonyFounded {
        name: String,
        index: usize,
    },
    ColonyDestroyed {
        name: String,
        index: usize,
        player: usize,
    },
    StarExplored {
        star_name: String,
        color_condition: bool,
        size_condition: bool,
        index: usize,
    },
    Story {
        title: String,
        details: String,
        order: u32,
        index: Option<usize>,
    },
    Fight {
        index: usize,
        star_name: String,
        against: usize,
        attacker: bool,
        ship_lost: u32,
        ship_destroyed: u32,
        population_killed: f32,
    },
}

impl Message {
    fn order(&self) -> u32 {
        match self {
            Message::Turn(_) => 0,
            Message::StarExplored { .. } => 1,
            Message::Fight { .. } => 2,
            Message::ColonyFounded { .. } => 3,
            Message::ColonyDestroyed { .. } => 4,
            Message::Story { order, .. } => 5 + order,
        }
    }

    pub fn as_sections(&self, ui_handles: &UiAssets) -> Vec<TextSection> {
        match self {
            Message::Turn(n) => vec![TextSection {
                value: format!("Turn {}", n),
                style: TextStyle {
                    font: ui_handles.font_main.clone_weak(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            }],
            Message::ColonyFounded { name, .. } => vec![TextSection {
                value: format!("Colony founded\non {}", name),
                style: TextStyle {
                    font: ui_handles.font_main.clone_weak(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            }],
            Message::ColonyDestroyed { name, player, .. } => vec![
                TextSection {
                    value: "Colony destroyed".to_string(),
                    style: TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: format!(
                        "{} destroyed you colony\non {}",
                        PLAYER_NAMES[*player], name
                    ),
                    style: TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
            Message::StarExplored {
                star_name,
                color_condition,
                size_condition,
                ..
            } => vec![
                TextSection {
                    value: format!("Star explored\n{}\n", star_name),
                    style: TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: if *size_condition {
                        "Star size condition: ideal\n".to_string()
                    } else {
                        "Star size condition: imperfect\n".to_string()
                    },
                    style: TextStyle {
                        font: ui_handles.font_sub.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: if *color_condition {
                        "Star color condition: ideal\n".to_string()
                    } else {
                        "Star color condition: bad\n".to_string()
                    },
                    style: TextStyle {
                        font: ui_handles.font_sub.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
            Message::Story { title, details, .. } => vec![
                TextSection {
                    value: format!("{}\n", title),
                    style: TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: details.clone(),
                    style: TextStyle {
                        font: ui_handles.font_sub.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
            Message::Fight {
                star_name,
                against,
                attacker,
                ship_lost,
                ship_destroyed,
                population_killed,
                ..
            } => vec![
                TextSection {
                    value: format!("Fight on {}\n", star_name),
                    style: TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: if *attacker {
                        format!("You attacked {}\nand lost {} ships.\nYou destroyed {} ships and\nkilled {:.1} population.", PLAYER_NAMES[*against], ship_lost, ship_destroyed, population_killed)
                    } else {
                        format!("You defended against {},\n lost {} ships and {:.1} population.\nYou destroyed {} ships.", PLAYER_NAMES[*against], ship_lost, population_killed, ship_destroyed)
                    },
                    style: TextStyle {
                        font: ui_handles.font_sub.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
        }
    }
}

struct FightReport {
    against: usize,
    attacker: bool,
    ship_lost: u32,
    ship_destroyed: u32,
    population_killed: f32,
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state(TurnState::Out)
            .add_system_set(SystemSet::on_enter(TurnState::Player).with_system(start_player_turn))
            .add_system_set(SystemSet::on_update(TurnState::Bots).with_system(run_bots_turn))
            .add_system_set(SystemSet::on_update(TurnState::Enemy).with_system(run_enemy_turn));
    }
}

#[allow(clippy::type_complexity)]
fn update_mask_for_star(
    star: usize,
    owner: usize,
    decorations: &mut ParamSet<(
        Query<(&mut Visibility, &StarHat)>,
        Query<(&mut Visibility, &mut Sprite, &StarMask)>,
    )>,
) {
    let mut p1 = decorations.p1();
    let mut p11 = p1.iter_mut();
    let (mut visibility, mut sprite, _) = p11.find(|(_, _, mask)| mask.0 == star).unwrap();
    if owner != 0 {
        visibility.is_visible = true;
        sprite.color = match owner {
            1 => Color::RED,
            2 => Color::BLUE,
            3 => Color::PURPLE,
            4 => Color::BLACK,
            _ => unreachable!(),
        };
    } else {
        visibility.is_visible = false;
    }
}

#[allow(clippy::type_complexity)]
fn start_player_turn(
    mut commands: Commands,
    mut universe: ResMut<Universe>,
    mut turns: ResMut<Turns>,
    galaxy_assets: Res<GalaxyAssets>,
    mut fleets: Query<(Entity, &mut Order, &Ship, &Owner, &mut FleetSize)>,
    mut materials: Query<&mut Handle<ColorMaterial>>,
    mut decorations: ParamSet<(
        Query<(&mut Visibility, &StarHat)>,
        Query<(&mut Visibility, &mut Sprite, &StarMask)>,
    )>,
) {
    turns.messages = vec![];

    if turns.count != 0 {
        let good_conditions = &universe.galaxy[universe.players[0].start].clone();

        for i in 0..universe.players.len() {
            universe.players[i].savings += universe.player_revenue(i);
        }
        let mut harvested = 0.0;
        universe
            .galaxy
            .clone()
            .iter()
            .zip(universe.star_details.iter_mut())
            .filter(|(_, details)| details.owner == 0)
            .for_each(|(star, details)| {
                // grow population
                {
                    let max_population = if star.color == good_conditions.color {
                        120.0 + (turns.count as f32 - details.owned_since as f32) / 5.0
                    } else {
                        10.0 + (turns.count as f32 - details.owned_since as f32) / 10.0
                    };
                    let lerp = details.population / max_population;
                    let growth_factor = if lerp < 0.5 {
                        (10.0 * lerp).powf(3.0)
                    } else if lerp < 1.0 {
                        1.0 - (-2.0 * lerp + 2.0).powf(3.0) / 2.0
                    } else {
                        1.0 - (-2.0 * lerp + 4.0).powf(3.0) / 2.0
                    };
                    details.population = if star.size == good_conditions.size {
                        details.population + growth_factor
                    } else {
                        details.population + growth_factor / 2.0
                    };
                }

                // harvest resources
                {
                    let to_get: f32 = if star.color == good_conditions.color {
                        0.3
                    } else {
                        1.0
                    };
                    let current_resources = (details.resources * 1.2).powf(1.5);
                    let collect = to_get.min(current_resources);
                    harvested += collect;
                    details.resources = if star.color != good_conditions.color {
                        ((details.resources * 1.2).powf(1.5) - collect).powf(1.0 / 1.5) / 1.2
                    } else {
                        ((details.resources).powf(0.8) - collect).powf(1.0 / 0.8)
                    }
                    .max(0.0);
                }
            });
        universe.players[0].resources += harvested;
    }

    let mut fleets_per_star: HashMap<usize, Vec<_>> = fleets.iter().fold(
        HashMap::new(),
        |mut acc, (_, order, ship, owner, fleet_size)| {
            match order {
                Order::Orbit(around) => {
                    acc.entry(*around)
                        .or_default()
                        .push((*owner, *ship, fleet_size.0 as i32))
                }
                Order::Move { from, to, step, .. } => {
                    if *step + 1
                        == turns_between(
                            universe.galaxy[*from].position,
                            universe.galaxy[*to].position,
                        )
                    {
                        acc.entry(*to)
                            .or_default()
                            .push((*owner, *ship, fleet_size.0 as i32))
                    }
                }
            };
            acc
        },
    );

    let mut fight_reports_per_star = HashMap::new();

    for (entity, mut order, ship, owner, mut fleet_size) in &mut fleets {
        match order.bypass_change_detection() {
            Order::Orbit(around) => match ship.kind {
                ShipKind::Colony => {
                    if let Some(other_owner) = fleets_per_star.get(around).and_then(|entry| {
                        entry
                            .iter()
                            .map(|(other_owner, _, _)| other_owner)
                            .find(|other_owner| other_owner.0 != owner.0)
                    }) {
                        // There is an ennemy ship, colony ships always lose
                        commands.entity(entity).despawn_recursive();
                        if owner.0 == 0 {
                            fight_reports_per_star
                                .entry(*around)
                                .or_insert(FightReport {
                                    against: other_owner.0,
                                    attacker: false,
                                    ship_lost: 0,
                                    ship_destroyed: 0,
                                    population_killed: 0.0,
                                })
                                .ship_lost += fleet_size.0;
                        } else if other_owner.0 == 0 {
                            fight_reports_per_star
                                .entry(*around)
                                .or_insert(FightReport {
                                    against: owner.0,
                                    attacker: true,
                                    ship_lost: 0,
                                    ship_destroyed: 0,
                                    population_killed: 0.0,
                                })
                                .ship_destroyed += fleet_size.0;
                        }
                        // no visibility to update here
                    }
                }
                ShipKind::Fighter => {
                    let enemy_fighters = fleets_per_star
                        .get(around)
                        .map(|entry| {
                            entry
                                .iter()
                                .filter(|(other_owner, ship, _)| {
                                    other_owner.0 != owner.0 && ship.kind == ShipKind::Fighter
                                })
                                .fold(
                                    vec![0, 0, 0, 0, 0],
                                    |mut acc, (other_owner, _, fleet_size)| {
                                        acc[other_owner.0] += fleet_size;
                                        acc
                                    },
                                )
                        })
                        .unwrap_or_else(|| vec![0; 5]);

                    for (u, n) in enemy_fighters.iter().enumerate() {
                        if fleet_size.0 > 0 {
                            let current_lost = fleet_size.0.min(*n as u32);
                            fleet_size.0 -= current_lost;
                            fleets_per_star.get_mut(around).unwrap().push((
                                Owner(u),
                                Ship {
                                    kind: ShipKind::Fighter,
                                },
                                -(current_lost as i32),
                            ));
                            if *n > 0 {
                                if owner.0 == 0 {
                                    let mut report = fight_reports_per_star
                                        .entry(*around)
                                        .or_insert(FightReport {
                                            against: u,
                                            attacker: false,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        });
                                    report.ship_lost += current_lost;
                                } else if u == 0 {
                                    let mut report = fight_reports_per_star
                                        .entry(*around)
                                        .or_insert(FightReport {
                                            against: owner.0,
                                            attacker: true,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        });
                                    report.ship_destroyed += current_lost;
                                }
                            }
                        }
                    }
                    if fleet_size.0 == 0 {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            },
            Order::Move { from, to, step, .. } => {
                *step += 1;
                if *step
                    == turns_between(
                        universe.galaxy[*from].position,
                        universe.galaxy[*to].position,
                    )
                {
                    // exploration
                    if universe.star_details[*to].owner != owner.0 {
                        // star exploration and visibility in universe
                        if owner.0 == 0 {
                            if universe.players[owner.0].vision[*to] == StarState::Unknown {
                                let start_conditions = &universe.galaxy[universe.players[0].start];
                                let new_star = &universe.galaxy[*to];
                                turns.messages.push(Message::StarExplored {
                                    star_name: universe.galaxy[*to].name.clone(),
                                    color_condition: start_conditions.color == new_star.color,
                                    size_condition: start_conditions.size == new_star.size,
                                    index: *to,
                                });
                            }

                            *materials.get_mut(universe.star_entities[*to]).unwrap() =
                                match universe.galaxy[*to].color {
                                    StarColor::Blue => galaxy_assets.blue_star.clone_weak(),
                                    StarColor::Orange => galaxy_assets.orange_star.clone_weak(),
                                    StarColor::Yellow => galaxy_assets.yellow_star.clone_weak(),
                                };
                        }

                        universe.players[owner.0].vision[*to] = StarState::Uninhabited;
                        if owner.0 == 0 {
                            update_mask_for_star(*to, 0, &mut decorations);
                        }
                    }
                    match ship.kind {
                        ShipKind::Colony => {
                            // fight against fleets
                            if let Some(other_owner) = fleets_per_star.get(to).and_then(|entry| {
                                entry
                                    .iter()
                                    .map(|(other_owner, _, _)| other_owner)
                                    .find(|other_owner| other_owner.0 != owner.0)
                            }) {
                                // There is an ennemy ship, colony ships always lose
                                commands.entity(entity).despawn_recursive();
                                universe.players[owner.0].vision[*to] =
                                    StarState::Owned(other_owner.0);

                                if owner.0 == 0 {
                                    update_mask_for_star(*to, other_owner.0, &mut decorations);
                                    fight_reports_per_star
                                        .entry(*to)
                                        .or_insert(FightReport {
                                            against: other_owner.0,
                                            attacker: true,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        })
                                        .ship_lost += 1;
                                } else if other_owner.0 == 0 {
                                    fight_reports_per_star
                                        .entry(*to)
                                        .or_insert(FightReport {
                                            against: owner.0,
                                            attacker: false,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        })
                                        .ship_destroyed += 1;
                                }
                                // ship destroyed, continue with next ship
                                continue;
                            }

                            // fight against population
                            if universe.star_details[*to].owner != owner.0
                                && universe.star_details[*to].owner != usize::MAX
                            {
                                // fight against population, colony ship always lose
                                commands.entity(entity).despawn_recursive();
                                universe.players[owner.0].vision[*to] =
                                    StarState::Owned(universe.star_details[*to].owner);

                                if owner.0 == 0 {
                                    update_mask_for_star(
                                        *to,
                                        universe.star_details[*to].owner,
                                        &mut decorations,
                                    );
                                    fight_reports_per_star
                                        .entry(*to)
                                        .or_insert(FightReport {
                                            against: universe.star_details[*to].owner,
                                            attacker: true,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        })
                                        .ship_lost += 1;
                                } else if universe.star_details[*to].owner == 0 {
                                    fight_reports_per_star
                                        .entry(*to)
                                        .or_insert(FightReport {
                                            against: owner.0,
                                            attacker: false,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        })
                                        .ship_destroyed += 1;
                                }

                                // ship destroyed, continue with next ship
                                continue;
                            }

                            // colonize the star!
                            if universe.star_details[*to].owner != owner.0 {
                                if owner.0 == 0 {
                                    turns.messages.push(Message::ColonyFounded {
                                        name: universe.galaxy[*to].name.clone(),
                                        index: *to,
                                    });
                                    if !universe.players[0].first_colony_done {
                                        universe.players[0].first_colony_done = true;
                                        turns.messages.push(Message::Story {
                                            title: "First colony!".to_string(),
                                            details: r#"You just founded your first colony!
If the color is the same as your
starting system, your population
will grow faster, but you'll
get less resources."#
                                                .to_string(),
                                            order: 0,
                                            index: None,
                                        });
                                        turns.messages.push(Message::Story {
                                            title: "revenue".to_string(),
                                            details: r#"New colonies cost credits.
Once population has grown, colonies
will start earning credits."#
                                                .to_string(),
                                            order: 1,
                                            index: None,
                                        });
                                    }

                                    decorations
                                        .p0()
                                        .iter_mut()
                                        .find(|(_, hat)| hat.0 == *to)
                                        .unwrap()
                                        .0
                                        .is_visible = true;
                                }

                                universe.players[owner.0].vision[*to] = StarState::Owned(owner.0);
                                universe.star_details[*to].owner = owner.0;
                                universe.star_details[*to].owned_since = turns.count;
                                universe.star_details[*to].population = 10.0;
                            }
                        }
                        ShipKind::Fighter => {
                            let enemy_fighters = fleets_per_star
                                .get(to)
                                .map(|entry| {
                                    entry
                                        .iter()
                                        .filter(|(other_owner, ship, _)| {
                                            other_owner.0 != owner.0
                                                && ship.kind == ShipKind::Fighter
                                        })
                                        .fold(
                                            vec![0, 0, 0, 0, 0],
                                            |mut acc, (other_owner, _, fleet_size)| {
                                                acc[other_owner.0] += fleet_size;
                                                acc
                                            },
                                        )
                                })
                                .unwrap_or_else(|| vec![0; 5]);

                            for (u, n) in enemy_fighters.iter().enumerate() {
                                if fleet_size.0 > 0 {
                                    let current_lost = fleet_size.0.min(*n as u32);
                                    fleet_size.0 -= current_lost;
                                    fleets_per_star.get_mut(to).unwrap().push((
                                        Owner(u),
                                        Ship {
                                            kind: ShipKind::Fighter,
                                        },
                                        -(current_lost as i32),
                                    ));
                                    if *n > 0 {
                                        if owner.0 == 0 {
                                            let mut report = fight_reports_per_star
                                                .entry(*to)
                                                .or_insert(FightReport {
                                                    against: u,
                                                    attacker: true,
                                                    ship_lost: 0,
                                                    ship_destroyed: 0,
                                                    population_killed: 0.0,
                                                });
                                            report.ship_lost += current_lost;
                                        } else if u == 0 {
                                            let mut report = fight_reports_per_star
                                                .entry(*to)
                                                .or_insert(FightReport {
                                                    against: owner.0,
                                                    attacker: false,
                                                    ship_lost: 0,
                                                    ship_destroyed: 0,
                                                    population_killed: 0.0,
                                                });
                                            report.ship_destroyed += current_lost;
                                        }
                                        if fleet_size.0 == 0 {
                                            commands.entity(entity).despawn_recursive();
                                            universe.players[owner.0].vision[*to] =
                                                StarState::Owned(u);
                                            if owner.0 == 0 {
                                                update_mask_for_star(*to, u, &mut decorations);
                                            }
                                            // ship destroyed, continue with next ship
                                            continue;
                                        }
                                    }
                                }
                            }
                            for (n, u) in enemy_fighters.iter().enumerate() {
                                if *u > 0 {
                                    // player n had ship on this star, and an enemy ship survived
                                    universe.players[n].vision[*to] = StarState::Owned(owner.0);
                                    if n == 0 {
                                        update_mask_for_star(*to, owner.0, &mut decorations);
                                    }
                                }
                            }
                            universe.players[owner.0].vision[*to] = StarState::Uninhabited;
                            if owner.0 == 0 {
                                update_mask_for_star(*to, 0, &mut decorations);
                            }

                            let attacked = universe.star_details[*to].owner;
                            if attacked != owner.0 && attacked != usize::MAX {
                                // fight against population, each fighter kills 10 population
                                let mut population = universe.star_details[*to].population;
                                let mut killed = 0.0;
                                let mut lost = 0;
                                while population > 10.0 && fleet_size.0 > 0 {
                                    population -= 10.0;
                                    killed += 10.0;
                                    fleet_size.0 -= 1;
                                    lost += 1;
                                }

                                if fleet_size.0 > 0 && population < 10.0 {
                                    // fleet is victorious
                                    killed = universe.star_details[*to].population;
                                    universe.star_details[*to].population = 0.0;
                                    universe.star_details[*to].owner = usize::MAX;
                                    universe.players[owner.0].vision[*to] = StarState::Uninhabited;
                                    universe.players[attacked].vision[*to] =
                                        StarState::Owned(owner.0);
                                    if owner.0 == 0 {
                                        update_mask_for_star(*to, 0, &mut decorations);
                                    } else if attacked == 0 {
                                        update_mask_for_star(*to, owner.0, &mut decorations);
                                        decorations
                                            .p0()
                                            .iter_mut()
                                            .find(|(_, hat)| hat.0 == *to)
                                            .unwrap()
                                            .0
                                            .is_visible = false;
                                        turns.messages.push(Message::ColonyDestroyed {
                                            name: universe.galaxy[*to].name.clone(),
                                            index: *to,
                                            player: owner.0,
                                        });
                                    }
                                } else {
                                    // fleet is destroyed
                                    commands.entity(entity).despawn_recursive();
                                    universe.star_details[*to].population = population;

                                    universe.players[owner.0].vision[*to] =
                                        StarState::Owned(attacked);
                                    if owner.0 == 0 {
                                        update_mask_for_star(*to, attacked, &mut decorations);
                                    }
                                }

                                if owner.0 == 0 {
                                    let mut report =
                                        fight_reports_per_star.entry(*to).or_insert(FightReport {
                                            against: attacked,
                                            attacker: true,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        });
                                    report.ship_lost += lost;
                                    report.population_killed += killed;
                                } else if attacked == 0 {
                                    let mut report =
                                        fight_reports_per_star.entry(*to).or_insert(FightReport {
                                            against: owner.0,
                                            attacker: false,
                                            ship_lost: 0,
                                            ship_destroyed: 0,
                                            population_killed: 0.0,
                                        });
                                    report.ship_destroyed += lost;
                                    report.population_killed += killed;
                                }
                            }
                        }
                    }

                    *order = Order::Orbit(*to);
                }
                order.set_changed();
            }
        }
    }

    for (index, fight_report) in fight_reports_per_star {
        turns.messages.push(Message::Fight {
            index,
            star_name: universe.galaxy[index].name.clone(),
            against: fight_report.against,
            attacker: fight_report.attacker,
            ship_lost: fight_report.ship_lost,
            ship_destroyed: fight_report.ship_destroyed,
            population_killed: fight_report.population_killed,
        });
    }

    let revenue = universe.player_revenue(0);
    if revenue < 0.0 {
        turns.messages.push(Message::Story {
            title: "Revenue Alert!".to_string(),
            details: "You have negative revenue.\nToo much debt and you'll lose\nthe game."
                .to_string(),
            order: 0,
            index: None,
        });
    }

    turns.count += 1;
    let count = turns.count;
    turns.messages.push(Message::Turn(count));
    if turns.count == 1 {
        turns.messages.push(Message::Story {
            title: "You can see your starting\nstar system".to_string(),
            details: "Click on it for more details.".to_string(),
            order: 0,
            index: None,
        });
        turns.messages.push(Message::Story {
            title: "Fleet panel".to_string(),
            details: "Drag and drop your colony ship\nto another star to launch it.".to_string(),
            order: 1,
            index: Some(universe.players[0].start),
        });
        turns.messages.push(Message::Story {
            title: "Ending turn".to_string(),
            details: r#"You can end your turn with the
button in the bottom right corner."#
                .to_string(),
            order: 1,
            index: None,
        });
        turns.messages.push(Message::Story {
            title: "Let's explore!".to_string(),
            details: "".to_string(),
            order: 4,
            index: None,
        });
    }
    turns.messages.sort_by_key(|m| m.order());
}

fn run_bots_turn(mut state: ResMut<State<TurnState>>) {
    let _ = state.set(TurnState::Enemy);
}

fn run_enemy_turn(mut state: ResMut<State<TurnState>>) {
    let _ = state.set(TurnState::Player);
}
