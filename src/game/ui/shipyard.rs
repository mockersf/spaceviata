use std::f32::consts::{FRAC_PI_8, PI};

use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    assets::{loader::ShipAssets, UiAssets},
    game::{
        fleet::{Fleet, FleetSize, Order, Owner, Ship, ShipKind},
        world::CameraControllerTarget,
        FleetsToSpawn, Universe,
    },
    ui_helper::button::ButtonId,
};

use super::{OneFrameDelay, ScreenTag, SelectedStar, DAMPENER};

#[derive(Clone, Copy)]
pub enum ShipyardButtons {
    BuildColonyShip,
    BuildFighter,
    Exit,
}

impl From<ShipyardButtons> for String {
    fn from(button: ShipyardButtons) -> Self {
        match button {
            ShipyardButtons::BuildColonyShip => {
                material_icons::icon_to_char(material_icons::Icon::Construction).to_string()
            }
            ShipyardButtons::BuildFighter => {
                material_icons::icon_to_char(material_icons::Icon::Construction).to_string()
            }
            ShipyardButtons::Exit => {
                material_icons::icon_to_char(material_icons::Icon::Logout).to_string()
            }
        }
    }
}

pub enum ShipyardEvent {
    OpenForStar(usize),
    Close,
    InsufficentSavings,
    InsufficentResources,
}

#[derive(Component)]
pub struct ShipyardPanelMarker;

#[derive(Component)]
pub struct ShipyardErrorMarker;

#[derive(Resource, Default)]
pub struct ShipyadForStar(usize);

pub fn display_shipyard(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    mut shipyard_events: EventReader<ShipyardEvent>,
    panel: Query<Entity, With<ShipyardPanelMarker>>,
    error: Query<Entity, With<ShipyardErrorMarker>>,
    mut target: ResMut<CameraControllerTarget>,
    mut selected_star: ResMut<SelectedStar>,
    mut for_star: ResMut<ShipyadForStar>,
    ship_assets: Res<ShipAssets>,
) {
    match shipyard_events.iter().last() {
        Some(ShipyardEvent::OpenForStar(index)) => {
            for_star.0 = *index;
            target.ignore_movement = true;
            selected_star.index = None;
            let button_handle = ui_handles.button_handle.clone_weak();
            let button = buttons.get(&button_handle).unwrap();

            let build_colony_button = button.add_hidden_section(
                &mut commands,
                Val::Px(250.),
                Val::Px(40.),
                UiRect::all(Val::Undefined),
                vec![
                    TextSection {
                        value: material_icons::icon_to_char(material_icons::Icon::Construction)
                            .to_string(),
                        style: TextStyle {
                            font: ui_handles.font_material.clone_weak(),
                            font_size: 15.0,
                            color: crate::ui_helper::ColorScheme::TEXT,
                        },
                    },
                    TextSection {
                        value: " 1 colony ship".to_string(),
                        style: TextStyle {
                            font: ui_handles.font_main.clone_weak(),
                            font_size: 20.0,
                            color: crate::ui_helper::ColorScheme::TEXT,
                        },
                    },
                ],
                ShipyardButtons::BuildColonyShip,
                20.,
                true,
            );
            let build_fighter_button = button.add_hidden_section(
                &mut commands,
                Val::Px(250.),
                Val::Px(40.),
                UiRect::all(Val::Undefined),
                vec![
                    TextSection {
                        value: material_icons::icon_to_char(material_icons::Icon::Construction)
                            .to_string(),
                        style: TextStyle {
                            font: ui_handles.font_material.clone_weak(),
                            font_size: 15.0,
                            color: crate::ui_helper::ColorScheme::TEXT,
                        },
                    },
                    TextSection {
                        value: " 1 fighter".to_string(),
                        style: TextStyle {
                            font: ui_handles.font_main.clone_weak(),
                            font_size: 20.0,
                            color: crate::ui_helper::ColorScheme::TEXT,
                        },
                    },
                ],
                ShipyardButtons::BuildFighter,
                20.,
                true,
            );

            let exit_button = button.add_hidden(
                &mut commands,
                Val::Px(30.),
                Val::Px(30.),
                UiRect::all(Val::Auto),
                ui_handles.font_material.clone_weak(),
                ShipyardButtons::Exit,
                20.,
                crate::ui_helper::ColorScheme::TEXT,
                true,
            );
            let base = commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            size: Size {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                            },
                            display: Display::None,
                            ..Default::default()
                        },

                        ..Default::default()
                    },
                    OneFrameDelay,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        text: Text::from_section(
                                            format!(
                                                r#"Build 1 {}
  credits: {}
  resources: {}"#,
                                                ShipKind::Colony,
                                                ShipKind::Colony.cost_credits(),
                                                ShipKind::Colony.cost_resources()
                                            ),
                                            TextStyle {
                                                font: ui_handles.font_sub.clone_weak(),
                                                font_size: 20.0,
                                                color: Color::WHITE,
                                            },
                                        ),
                                        style: Style {
                                            size: Size {
                                                width: Val::Px(200.0),
                                                height: Val::Px(70.0),
                                            },
                                            ..default()
                                        },
                                        ..default()
                                    });
                                    parent.spawn(ImageBundle {
                                        image: UiImage(ship_assets.colony_ship.clone_weak()),
                                        style: Style {
                                            size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                                            ..default()
                                        },
                                        transform: Transform::from_rotation(Quat::from_rotation_z(
                                            FRAC_PI_8 + PI,
                                        )),
                                        ..default()
                                    });
                                });
                        })
                        .push_children(&[build_colony_button]);
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::all(Val::Px(10.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn(TextBundle {
                                        text: Text::from_section(
                                            format!(
                                                r#"Build 1 {}
  credits: {}
  resources: {}"#,
                                                ShipKind::Fighter,
                                                ShipKind::Fighter.cost_credits(),
                                                ShipKind::Fighter.cost_resources()
                                            ),
                                            TextStyle {
                                                font: ui_handles.font_sub.clone_weak(),
                                                font_size: 20.0,
                                                color: Color::WHITE,
                                            },
                                        ),
                                        style: Style {
                                            size: Size {
                                                width: Val::Px(200.0),
                                                height: Val::Px(70.0),
                                            },
                                            ..default()
                                        },
                                        ..default()
                                    });
                                    parent.spawn(ImageBundle {
                                        image: UiImage(ship_assets.fighter.clone_weak()),
                                        style: Style {
                                            size: Size::new(Val::Px(50.0), Val::Px(50.0)),
                                            ..default()
                                        },
                                        transform: Transform::from_rotation(Quat::from_rotation_z(
                                            FRAC_PI_8 + PI,
                                        )),
                                        ..default()
                                    });
                                });
                        })
                        .push_children(&[build_fighter_button]);
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: UiRect {
                                    bottom: Val::Px(0.0),
                                    ..default()
                                },
                                size: Size::new(Val::Percent(100.0), Val::Px(30.0)),
                                display: Display::None,
                                ..default()
                            },
                            ..default()
                        },
                        ShipyardErrorMarker,
                        OneFrameDelay,
                    ));
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        right: Val::Px(0.0),
                                        bottom: Val::Px(0.0),
                                        ..default()
                                    },
                                    display: Display::None,
                                    ..default()
                                },
                                ..default()
                            },
                            OneFrameDelay,
                        ))
                        .push_children(&[exit_button]);
                })
                .id();

            let panel_style = Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                size: Size::new(Val::Px(300.0), Val::Px(400.0)),
                align_content: AlignContent::Stretch,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            };

            let message_panel = commands
                .spawn(bevy_ninepatch::NinePatchBundle {
                    style: panel_style,
                    nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                        ui_handles.panel_handle.1.clone_weak(),
                        ui_handles.panel_handle.0.clone_weak(),
                        base,
                    ),
                    ..default()
                })
                .id();

            commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Auto),
                            size: Size::new(Val::Px(600.0), Val::Px(600.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        z_index: ZIndex::Global(10),
                        ..default()
                    },
                    ShipyardPanelMarker,
                    ScreenTag,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(DAMPENER),
                            ..default()
                        })
                        .push_children(&[message_panel]);
                });
        }
        Some(ShipyardEvent::Close) => {
            if let Ok(entity) = panel.get_single() {
                target.ignore_movement = false;
                commands.entity(entity).despawn_recursive();
            }
        }
        Some(ShipyardEvent::InsufficentSavings) => {
            if let Ok(entity) = error.get_single() {
                commands.entity(entity).despawn_descendants();
                commands.entity(entity).with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Insufficient Savings",
                            TextStyle {
                                font: ui_handles.font_sub.clone_weak(),
                                font_size: 20.0,
                                color: Color::rgb(0.64, 0.17, 0.17),
                            },
                        ),
                        ..default()
                    });
                });
            }
        }
        Some(ShipyardEvent::InsufficentResources) => {
            if let Ok(entity) = error.get_single() {
                commands.entity(entity).despawn_descendants();
                commands.entity(entity).with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Insufficient Resources",
                            TextStyle {
                                font: ui_handles.font_sub.clone_weak(),
                                font_size: 20.0,
                                color: Color::ORANGE_RED,
                            },
                        ),
                        ..default()
                    });
                });
            }
        }
        None => (),
    }
}

pub fn button_system(
    interaction_query: Query<(&Interaction, &ButtonId<ShipyardButtons>), Changed<Interaction>>,
    mut shipyard_events: EventWriter<ShipyardEvent>,
    mut universe: ResMut<Universe>,
    mut fleets_to_spawn: ResMut<FleetsToSpawn>,
    for_star: Res<ShipyadForStar>,
) {
    for (interaction, button_id) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button_id.0 {
                ShipyardButtons::Exit => shipyard_events.send(ShipyardEvent::Close),
                ShipyardButtons::BuildColonyShip => {
                    if universe.players[0].savings < ShipKind::Colony.cost_credits() {
                        shipyard_events.send(ShipyardEvent::InsufficentSavings);
                        return;
                    }
                    if universe.players[0].resources < ShipKind::Colony.cost_resources() {
                        shipyard_events.send(ShipyardEvent::InsufficentResources);
                        return;
                    }
                    universe.players[0].savings -= ShipKind::Colony.cost_credits();
                    universe.players[0].resources -= ShipKind::Colony.cost_resources();
                    fleets_to_spawn.0.push(Fleet {
                        order: Order::Orbit(for_star.0),
                        ship: Ship {
                            kind: ShipKind::Colony,
                        },
                        size: FleetSize(1),
                        owner: Owner(0),
                    });
                    shipyard_events.send(ShipyardEvent::Close);
                }
                ShipyardButtons::BuildFighter => {
                    if universe.players[0].savings < ShipKind::Fighter.cost_credits() {
                        shipyard_events.send(ShipyardEvent::InsufficentSavings);
                        return;
                    }
                    if universe.players[0].resources < ShipKind::Fighter.cost_resources() {
                        shipyard_events.send(ShipyardEvent::InsufficentResources);
                        return;
                    }
                    universe.players[0].savings -= ShipKind::Fighter.cost_credits();
                    universe.players[0].resources -= ShipKind::Fighter.cost_resources();
                    fleets_to_spawn.0.push(Fleet {
                        order: Order::Orbit(for_star.0),
                        ship: Ship {
                            kind: ShipKind::Fighter,
                        },
                        size: FleetSize(1),
                        owner: Owner(0),
                    });
                    shipyard_events.send(ShipyardEvent::Close);
                }
            }
        }
    }
}
