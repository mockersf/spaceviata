use bevy::prelude::*;

use crate::assets::{GalaxyAssets, UiAssets};

use super::{
    fleet::{turns_between, Order, Owner, Ship},
    galaxy::StarColor,
    world::StarHat,
    StarState, Universe,
};

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub(crate) enum TurnState {
    Player,
    Bots,
    Enemy,
    Out,
}

#[derive(Resource)]
pub(crate) struct Turns {
    pub(crate) count: u32,
    pub(crate) messages: Vec<Message>,
}

pub(crate) enum Message {
    Turn(u32),
    ColonyFounded(String),
    StarExplored {
        star_name: String,
        color_condition: bool,
        size_condition: bool,
    },
    Story(String),
}

// impl ToString for Message {
//     fn to_string(&self) -> String {
//         match self {
//             Message::Turn(n) => format!("Turn {}", n),
//             Message::StarExplored {
//                 star_name,
//                 color_condition,
//                 size_condition,
//             } => format!("Just explored\n{}", star_name),
//             Message::ColonyFounded(star_name) => format!("Colony founded\non {}", star_name),
//             Message::Story(message) => message.clone(),
//         }
//     }
// }

impl Message {
    fn order(&self) -> u32 {
        match self {
            Message::Turn(_) => 0,
            Message::StarExplored { .. } => 1,
            Message::ColonyFounded(_) => 2,
            Message::Story(_) => 3,
        }
    }

    pub(crate) fn as_sections(&self, ui_handles: &UiAssets) -> Vec<TextSection> {
        match self {
            Message::Turn(n) => vec![TextSection {
                value: format!("Turn {}", n),
                style: TextStyle {
                    font: ui_handles.font_main.clone_weak(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            }],
            Message::ColonyFounded(name) => vec![TextSection {
                value: format!("Colony founded\non {}", name),
                style: TextStyle {
                    font: ui_handles.font_main.clone_weak(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            }],
            Message::StarExplored {
                star_name,
                color_condition,
                size_condition,
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
                        format!("Star size condition: ideal\n")
                    } else {
                        format!("Star size condition: imperfect\n")
                    },
                    style: TextStyle {
                        font: ui_handles.font_sub.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
                TextSection {
                    value: if *color_condition {
                        format!("Star color condition: ideal\n")
                    } else {
                        format!("Star color condition: bad\n")
                    },
                    style: TextStyle {
                        font: ui_handles.font_sub.clone_weak(),
                        font_size: 20.0,
                        color: Color::WHITE,
                    },
                },
            ],
            Message::Story(message) => vec![TextSection {
                value: message.clone(),
                style: TextStyle {
                    font: ui_handles.font_main.clone_weak(),
                    font_size: 20.0,
                    color: Color::WHITE,
                },
            }],
        }
    }
}
pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state(TurnState::Out)
            .add_system_set(SystemSet::on_enter(TurnState::Player).with_system(start_player_turn))
            .add_system_set(SystemSet::on_update(TurnState::Bots).with_system(run_bots_turn))
            .add_system_set(SystemSet::on_update(TurnState::Enemy).with_system(run_enemy_turn));
    }
}

fn start_player_turn(
    mut universe: ResMut<Universe>,
    mut turns: ResMut<Turns>,
    galaxy_assets: Res<GalaxyAssets>,
    mut fleets: Query<(&mut Order, &Ship, &Owner)>,
    mut materials: Query<&mut Handle<ColorMaterial>>,
    mut hats: Query<(&mut Visibility, &StarHat)>,
) {
    turns.messages = vec![];

    if turns.count != 0 {
        let good_conditions = &universe.galaxy[universe.players[0].start].clone();

        universe.players[0].savings += universe.player_revenue(0);
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
                        details.population + growth_factor / 5.0
                    };
                }

                // harvest resources
                {
                    let current_resources = (details.resources * 1.2).powf(1.5);
                    let collect = 1.0_f32.min(current_resources);
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

    for (mut order, ship, owner) in &mut fleets {
        match order.as_mut() {
            Order::Orbit(_) => (),
            Order::Move { from, to, step, .. } => {
                *step += 1;
                if *step
                    == turns_between(
                        universe.galaxy[*from].position,
                        universe.galaxy[*to].position,
                    )
                {
                    match ship.kind {
                        super::fleet::ShipKind::Colony => {
                            if universe.star_details[*to].owner != owner.0 {
                                if owner.0 == 0 {
                                    if universe.players[owner.0].vision[*to] == StarState::Unknown {
                                        let start_conditions =
                                            &universe.galaxy[universe.players[0].start];
                                        let new_star = &universe.galaxy[*to];
                                        turns.messages.push(Message::StarExplored {
                                            star_name: universe.galaxy[*to].name.clone(),
                                            color_condition: start_conditions.color
                                                == new_star.color,
                                            size_condition: start_conditions.size == new_star.size,
                                        });
                                    }
                                    turns.messages.push(Message::ColonyFounded(
                                        universe.galaxy[*to].name.clone(),
                                    ));
                                }
                                universe.players[owner.0].vision[*to] = StarState::Owned(owner.0);
                                universe.star_details[*to].owner = owner.0;
                                universe.star_details[*to].owned_since = turns.count;
                                universe.star_details[*to].population = 10.0;
                                *materials.get_mut(universe.star_entities[*to]).unwrap() =
                                    match universe.galaxy[*to].color {
                                        StarColor::Blue => galaxy_assets.blue_star.clone_weak(),
                                        StarColor::Orange => galaxy_assets.orange_star.clone_weak(),
                                        StarColor::Yellow => galaxy_assets.yellow_star.clone_weak(),
                                    };
                                if owner.0 == 0 {
                                    hats.iter_mut()
                                        .find(|(_, hat)| hat.0 == *to)
                                        .unwrap()
                                        .0
                                        .is_visible = true;
                                }
                            }
                        }
                    }

                    *order = Order::Orbit(*to);
                }
            }
        }
    }
    turns.count += 1;
    let count = turns.count;
    turns.messages.push(Message::Turn(count));
    if turns.count == 1 {
        turns
            .messages
            .push(Message::Story("Let's explore!".to_string()));
    }
    turns.messages.sort_by_key(|m| m.order());
}

fn run_bots_turn(mut state: ResMut<State<TurnState>>) {
    let _ = state.set(TurnState::Enemy);
}

fn run_enemy_turn(mut state: ResMut<State<TurnState>>) {
    let _ = state.set(TurnState::Player);
}
