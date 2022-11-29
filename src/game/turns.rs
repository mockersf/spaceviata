use bevy::prelude::*;

use crate::assets::{GalaxyAssets, UiAssets};

use super::{
    fleet::{turns_between, Order, Owner, Ship, ShipKind},
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
    ColonyFounded {
        name: String,
        index: usize,
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
}

impl Message {
    fn order(&self) -> u32 {
        match self {
            Message::Turn(_) => 0,
            Message::StarExplored { .. } => 1,
            Message::ColonyFounded { .. } => 2,
            Message::Story { order, .. } => 3 + order,
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
            Message::ColonyFounded { name, .. } => vec![TextSection {
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
        match order.bypass_change_detection() {
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
                        ShipKind::Colony => {
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
                                            index: *to,
                                        });
                                    }
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
will grow faster."#
                                                .to_string(),
                                            order: 0,
                                            index: None,
                                        });
                                        turns.messages.push(Message::Story {
                                            title: "revenue".to_string(),
                                            details: r#"New colony cost credits.
Once population has grown, colonies
will start earning credits."#
                                                .to_string(),
                                            order: 1,
                                            index: None,
                                        });
                                    }
                                    *materials.get_mut(universe.star_entities[*to]).unwrap() =
                                        match universe.galaxy[*to].color {
                                            StarColor::Blue => galaxy_assets.blue_star.clone_weak(),
                                            StarColor::Orange => {
                                                galaxy_assets.orange_star.clone_weak()
                                            }
                                            StarColor::Yellow => {
                                                galaxy_assets.yellow_star.clone_weak()
                                            }
                                        };
                                    if owner.0 == 0 {
                                        hats.iter_mut()
                                            .find(|(_, hat)| hat.0 == *to)
                                            .unwrap()
                                            .0
                                            .is_visible = true;
                                    }
                                }
                                universe.players[owner.0].vision[*to] = StarState::Owned(owner.0);
                                universe.star_details[*to].owner = owner.0;
                                universe.star_details[*to].owned_since = turns.count;
                                universe.star_details[*to].population = 10.0;
                            }
                        }
                        ShipKind::Fighter => {
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
                                            index: *to,
                                        });
                                    }
                                    *materials.get_mut(universe.star_entities[*to]).unwrap() =
                                        match universe.galaxy[*to].color {
                                            StarColor::Blue => galaxy_assets.blue_star.clone_weak(),
                                            StarColor::Orange => {
                                                galaxy_assets.orange_star.clone_weak()
                                            }
                                            StarColor::Yellow => {
                                                galaxy_assets.yellow_star.clone_weak()
                                            }
                                        };
                                }
                                universe.players[owner.0].vision[*to] = StarState::Uninhabited;
                            }
                        }
                    }

                    *order = Order::Orbit(*to);
                }
                order.set_changed();
            }
        }
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
