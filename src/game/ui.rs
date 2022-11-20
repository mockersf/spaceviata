use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
};

use crate::{assets::UiAssets, ui_helper::button::ButtonId, GameState};

use super::{
    galaxy::StarSize,
    world::{CameraController, CameraControllerTarget, RATIO_ZOOM_DISTANCE},
    StarState, Universe,
};

pub const LEFT_PANEL_WIDTH: f32 = 200.0;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(button_system)
                    .with_system(select_star)
                    .with_system(display_star_list)
                    .with_system(star_list_scroll),
            )
            .add_system_set(SystemSet::on_exit(GameState::Game).with_system(tear_down));
    }
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Clone, Copy, PartialEq)]
enum UiButtons {
    ZoomIn,
    ZoomOut,
    GameMenu,
    BackToMenu,
}

#[derive(Component)]
struct ScreenTag;

impl From<UiButtons> for String {
    fn from(button: UiButtons) -> Self {
        match button {
            UiButtons::ZoomIn => {
                material_icons::icon_to_char(material_icons::Icon::ZoomIn).to_string()
            }
            UiButtons::ZoomOut => {
                material_icons::icon_to_char(material_icons::Icon::ZoomOut).to_string()
            }
            UiButtons::GameMenu => {
                material_icons::icon_to_char(material_icons::Icon::Settings).to_string()
            }
            UiButtons::BackToMenu => "Menu".to_string(),
        }
    }
}

#[derive(Component)]
struct LiveMarker;

#[derive(Component)]
struct CreditsMarker;

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
) {
    info!("loading UI");

    let button_handle = ui_handles.button_handle.clone_weak();
    let button = buttons.get(&button_handle).unwrap();
    let material = ui_handles.font_material.clone_weak();

    let zoom_in_button = button.add(
        &mut commands,
        Val::Px(40.),
        Val::Px(40.),
        UiRect::all(Val::Auto),
        material.clone(),
        UiButtons::ZoomIn,
        25.,
        crate::ui_helper::ColorScheme::TEXT,
    );
    let zoom_out_button = button.add(
        &mut commands,
        Val::Px(40.),
        Val::Px(40.),
        UiRect::all(Val::Auto),
        material.clone(),
        UiButtons::ZoomOut,
        25.,
        crate::ui_helper::ColorScheme::TEXT,
    );
    let game_menu_button = button.add(
        &mut commands,
        Val::Px(40.),
        Val::Px(40.),
        UiRect::all(Val::Auto),
        material,
        UiButtons::GameMenu,
        25.,
        crate::ui_helper::ColorScheme::TEXT,
    );
    let back_to_menu_button = button.add(
        &mut commands,
        Val::Px(100.),
        Val::Px(40.),
        UiRect::all(Val::Auto),
        ui_handles.font_sub.clone_weak(),
        UiButtons::BackToMenu,
        25.,
        crate::ui_helper::ColorScheme::TEXT,
    );

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    ..default()
                },
                ..default()
            },
            ScreenTag,
        ))
        .with_children(|commands| {
            commands
                .spawn(NodeBundle {
                    style: Style {
                        position: UiRect {
                            right: Val::Px(20.0),
                            top: Val::Px(20.0),
                            ..default()
                        },
                        size: Size {
                            width: Val::Px(100.0),
                            height: Val::Px(150.0),
                        },
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            ..default()
                        })
                        .push_children(&[zoom_in_button, zoom_out_button]);
                })
                .push_children(&[game_menu_button, back_to_menu_button])
                .with_children(|builder| {
                    builder
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Column,
                                    justify_content: JustifyContent::SpaceAround,
                                    ..default()
                                },
                                visibility: Visibility::INVISIBLE,
                                ..default()
                            },
                            MenuContainer,
                        ))
                        .push_children(&[back_to_menu_button]);
                });
        });

    let base = commands
        .spawn((
            NodeBundle {
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
            },
            ScreenTag,
        ))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        max_size: Size::UNDEFINED,
                        ..default()
                    },
                    ..default()
                },
                StarList::default(),
            ));
        })
        .id();

    let panel_style = Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            left: Val::Px(0.),
            right: Val::Undefined,
            bottom: Val::Undefined,
            top: Val::Undefined,
        },
        margin: UiRect::all(Val::Px(0.)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        size: Size::new(Val::Px(LEFT_PANEL_WIDTH), Val::Percent(100.0)),
        align_content: AlignContent::Stretch,
        flex_direction: FlexDirection::Column,
        ..Default::default()
    };

    commands.spawn((
        bevy_ninepatch::NinePatchBundle {
            style: panel_style,
            nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                ui_handles.left_panel_handle.1.clone_weak(),
                ui_handles.left_panel_handle.0.clone_weak(),
                base,
            ),
            ..Default::default()
        },
        ScreenTag,
    ));
}

#[derive(Component, Default)]
struct StarList {
    position: f32,
}

#[derive(Component)]
struct MenuContainer;

fn button_system(
    interaction_query: Query<(&Interaction, &ButtonId<UiButtons>, Changed<Interaction>)>,
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    mut state: ResMut<State<GameState>>,
    mut menu_container: Query<&mut Visibility, With<MenuContainer>>,
) {
    for (interaction, button_id, changed) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match (button_id.0, changed) {
                (UiButtons::ZoomIn, _) => {
                    target.zoom_level = (controller.zoom_level + 1.0).min(10.0);
                    target.ignore_movement = true;
                }
                (UiButtons::ZoomOut, _) => {
                    target.zoom_level = (controller.zoom_level - 1.0).max(1.0);
                    target.ignore_movement = true;
                }
                (UiButtons::GameMenu, true) => {
                    menu_container.single_mut().toggle();
                }
                (UiButtons::BackToMenu, true) => state.set(GameState::Menu).unwrap(),
                _ => (),
            }
        }
        if *interaction == Interaction::None && changed {
            target.ignore_movement = false;
        }
    }
}

fn select_star(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
    universe: Res<Universe>,
    controller: Res<CameraController>,
) {
    if windows.primary().cursor_position().unwrap().x < LEFT_PANEL_WIDTH {
        return;
    }

    if mouse_input.just_pressed(MouseButton::Left) {
        let (camera, transform) = camera.single();
        let clicked = camera
            .viewport_to_world(transform, windows.primary().cursor_position().unwrap())
            .unwrap()
            .origin
            .xy();
        if let Some(clicked) = universe.galaxy.iter().find(|star| {
            (star.position * controller.zoom_level / RATIO_ZOOM_DISTANCE).distance(clicked)
                < <StarSize as Into<f32>>::into(star.size) * controller.zoom_level.powf(0.7) * 2.5
        }) {
            info!("{:?}", clicked);
        }
    }
}

fn display_star_list(
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
                parent.spawn(TextBundle {
                    text: Text::from_section(
                        universe.galaxy[star].name.clone(),
                        TextStyle {
                            font: ui_assets.font_sub.clone_weak(),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    ),
                    style: Style {
                        size: Size {
                            width: Val::Undefined,
                            height: Val::Px(20.0),
                        },
                        flex_shrink: 0.,
                        ..default()
                    },
                    ..default()
                });
            }
        });
    }
}

fn star_list_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut StarList, &mut Style, &Children, &Node)>,
    query_item: Query<&Node>,
    windows: Res<Windows>,
) {
    if windows.primary().cursor_position().unwrap().x > LEFT_PANEL_WIDTH {
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
