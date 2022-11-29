use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    ui::FocusPolicy,
};

use crate::{
    assets::UiAssets,
    game::{world::CameraControllerTarget, StarState, Universe},
};

use super::{shipyard, ScreenTag, SelectedStar, DAMPENER, LEFT_PANEL_WIDTH};

#[derive(Component)]
pub struct PlayerStatsMarker;

#[derive(Component, Default)]
pub struct StarList {
    position: f32,
}

pub fn setup(commands: &mut Commands, ui_handles: &UiAssets) {
    let left_panel_top = {
        let base = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Px(10.0)),
                    size: Size {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                    },
                    overflow: Overflow::Hidden,
                    ..Default::default()
                },
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn((
                    TextBundle {
                        text: Text::from_sections([
                            TextSection {
                                value: "Population ".to_string(),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: format!("{}\n", 0),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "Revenue    ".to_string(),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: format!("{}\n", 0),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "Savings    ".to_string(),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: format!("{}\n", 0),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: "Resources  ".to_string(),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                            TextSection {
                                value: format!("{}\n", 0),
                                style: TextStyle {
                                    font: ui_handles.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            },
                        ]),
                        style: Style {
                            size: Size {
                                width: Val::Undefined,
                                height: Val::Px(80.0),
                            },
                            ..default()
                        },
                        ..default()
                    },
                    PlayerStatsMarker,
                ));
            })
            .id();

        let panel_height = 120.0;
        let panel_style = Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Percent(100.0), Val::Px(panel_height)),
            align_content: AlignContent::Stretch,
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0,
            min_size: Size::new(Val::Percent(100.0), Val::Px(panel_height)),
            max_size: Size::new(Val::Percent(100.0), Val::Px(panel_height)),
            ..Default::default()
        };

        commands
            .spawn(bevy_ninepatch::NinePatchBundle {
                style: panel_style,
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    ui_handles.tr_panel_handle.1.clone_weak(),
                    ui_handles.tr_panel_handle.0.clone_weak(),
                    base,
                ),
                ..default()
            })
            .id()
    };

    let left_panel_bottom = {
        let base = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Px(10.0)),
                    size: Size {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                    },
                    overflow: Overflow::Hidden,
                    ..Default::default()
                },
                focus_policy: FocusPolicy::Pass,
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            max_size: Size::UNDEFINED,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        ..default()
                    },
                    StarList::default(),
                ));
            })
            .id();

        let panel_style = Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            align_content: AlignContent::Stretch,
            flex_direction: FlexDirection::Column,
            flex_grow: 2.0,
            ..Default::default()
        };

        commands
            .spawn(bevy_ninepatch::NinePatchBundle {
                style: panel_style,
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    ui_handles.br_panel_handle.1.clone_weak(),
                    ui_handles.br_panel_handle.0.clone_weak(),
                    base,
                ),
                ..default()
            })
            .id()
    };

    let panel_style = Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            left: Val::Px(0.0),
            right: Val::Undefined,
            top: Val::Px(0.0),
            bottom: Val::Undefined,
        },
        margin: UiRect::all(Val::Px(0.)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        size: Size::new(Val::Px(LEFT_PANEL_WIDTH), Val::Percent(100.0)),
        align_content: AlignContent::Stretch,
        flex_direction: FlexDirection::Column,
        ..Default::default()
    };

    commands
        .spawn((
            NodeBundle {
                style: panel_style,
                background_color: BackgroundColor(DAMPENER),
                focus_policy: FocusPolicy::Pass,
                ..default()
            },
            ScreenTag,
        ))
        .push_children(&[left_panel_top, left_panel_bottom]);
}

pub fn update_player_stats(
    mut text: Query<&mut Text, With<PlayerStatsMarker>>,
    universe: Res<Universe>,
) {
    if universe.is_changed() {
        let mut text = text.single_mut();
        text.sections[1].value = format!("{:.1}\n", universe.player_population(0));
        let revenue = universe.player_revenue(0);
        text.sections[3].value = format!("{:.1}\n", universe.player_revenue(0));
        if revenue < 0.0 {
            text.sections[3].style.color = Color::rgb(0.64, 0.17, 0.17)
        } else {
            text.sections[3].style.color = Color::GREEN
        }
        text.sections[5].value = format!("{:.1}\n", universe.players[0].savings);
        if universe.players[0].savings < 0.0 {
            text.sections[5].style.color = Color::rgb(0.64, 0.17, 0.17)
        } else {
            text.sections[5].style.color = Color::GREEN
        }
        text.sections[7].value = format!("{:.1}\n", universe.players[0].resources);
    }
}

#[allow(clippy::type_complexity)]
pub fn star_list_click(
    interaction_query: Query<(&Interaction, &StarListIndex), (Changed<Interaction>, With<Button>)>,
    mut selected_star: ResMut<SelectedStar>,
    universe: Res<Universe>,
    mut controller_target: ResMut<CameraControllerTarget>,
    mut shipyard: ResMut<Events<shipyard::ShipyardEvent>>,
) {
    for (interaction, star_index) in &interaction_query {
        if *interaction == Interaction::Clicked {
            shipyard.send(shipyard::ShipyardEvent::Close);
            if selected_star.index == Some(star_index.0) {
                controller_target.zoom_level = 8.0;
                controller_target.position = universe.galaxy[star_index.0].position;
            } else {
                selected_star.index = Some(star_index.0);
            }
        }
    }
}

pub fn star_list_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut StarList, &mut Style, &Children, &Node)>,
    query_item: Query<&Node>,
    windows: Res<Windows>,
) {
    if windows.primary().cursor_position().is_none()
        || windows.primary().cursor_position().unwrap().x > LEFT_PANEL_WIDTH
    {
        return;
    }
    for mouse_wheel_event in mouse_wheel_events.iter() {
        for (mut scrolling_list, mut style, children, uinode) in &mut query_list {
            let items_height: f32 = children
                .iter()
                .map(|entity| query_item.get(*entity).unwrap().size().y)
                .sum();
            let panel_height = uinode.size().y;
            let max_scroll = items_height - panel_height.max(0.);
            if max_scroll > 0.0 {
                let dy = match mouse_wheel_event.unit {
                    MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                    MouseScrollUnit::Pixel => mouse_wheel_event.y,
                };
                scrolling_list.position += dy;
                scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
                style.position.top = Val::Px(scrolling_list.position);
            }
        }
    }
}

#[derive(Component)]
pub struct StarListIndex(usize);

pub fn display_star_list(
    mut commands: Commands,
    universe: Res<Universe>,
    ui_container: Query<(Entity, Option<&Children>), With<StarList>>,
    ui_assets: Res<UiAssets>,
) {
    let Ok(ui_container) = ui_container.get_single() else {
        return;
    };

    if universe.is_changed() || ui_container.1.is_some() {
        commands.entity(ui_container.0).despawn_descendants();
        commands.entity(ui_container.0).with_children(|parent| {
            for (star, _) in universe.players[0]
                .vision
                .iter()
                .enumerate()
                .filter(|(_, state)| **state == StarState::Owned(0))
            {
                parent
                    .spawn((
                        ButtonBundle {
                            background_color: BackgroundColor(Color::NONE),
                            style: Style {
                                size: Size {
                                    width: Val::Undefined,
                                    height: Val::Px(20.0),
                                },
                                flex_shrink: 0.,
                                ..default()
                            },
                            ..default()
                        },
                        StarListIndex(star),
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle {
                            text: Text::from_section(
                                universe.galaxy[star]
                                    .name
                                    .split(' ')
                                    .map(|word| {
                                        let mut chars = word.chars();
                                        chars.next().unwrap().to_uppercase().collect::<String>()
                                            + chars.as_str()
                                    })
                                    .collect::<Vec<String>>()
                                    .join(" "),
                                TextStyle {
                                    font: ui_assets.font_sub.clone_weak(),
                                    font_size: 20.0,
                                    color: Color::WHITE,
                                },
                            ),
                            ..default()
                        });
                    });
            }
        });
    }
}
