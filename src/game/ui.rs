use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
};
use bevy_prototype_lyon::{
    prelude::{DrawMode, GeometryBuilder, StrokeMode},
    shapes,
};

use crate::{assets::UiAssets, ui_helper::button::ButtonId, GameState};

use super::{
    galaxy::StarSize,
    world::{CameraController, CameraControllerTarget, RATIO_ZOOM_DISTANCE},
    z_levels, StarState, Universe,
};

pub const LEFT_PANEL_WIDTH: f32 = 200.0;

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SelectedStar>()
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(button_system)
                    .with_system(select_star)
                    .with_system(display_star_list)
                    .with_system(star_list_scroll)
                    .with_system(display_star_selected)
                    .with_system(rotate_mark),
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
        NodeBundle {
            style: panel_style.clone(),
            background_color: BackgroundColor(Color::rgba(0.5, 0.5, 0.5, 0.75)),
            ..default()
        },
        ScreenTag,
    ));
    commands.spawn((
        bevy_ninepatch::NinePatchBundle {
            style: panel_style,
            nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                ui_handles.left_panel_handle.1.clone_weak(),
                ui_handles.left_panel_handle.0.clone_weak(),
                base,
            ),
            ..default()
        },
        ScreenTag,
    ));
}

#[derive(Component, Default)]
struct StarList {
    position: f32,
}

#[derive(Resource, Default)]
struct SelectedStar(Option<usize>);

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
    touches: Res<Touches>,
    windows: Res<Windows>,
    camera: Query<(&Camera, &GlobalTransform)>,
    universe: Res<Universe>,
    controller: Res<CameraController>,
    mut selected_star: ResMut<SelectedStar>,
) {
    if let Some(position) = mouse_input
        .just_pressed(MouseButton::Left)
        .then(|| windows.primary().cursor_position())
        .flatten()
        .or_else(|| {
            touches.first_pressed_position().map(|mut pos| {
                pos.y = windows.primary().height() - pos.y;
                pos
            })
        })
    {
        if position.x < LEFT_PANEL_WIDTH {
            return;
        }
        let (camera, transform) = camera.single();
        let clicked = camera
            .viewport_to_world(transform, position)
            .unwrap()
            .origin
            .xy();
        if let Some((index, _)) = universe.galaxy.iter().enumerate().find(|(_, star)| {
            (star.position * controller.zoom_level / RATIO_ZOOM_DISTANCE).distance(clicked)
                < <StarSize as Into<f32>>::into(star.size) * controller.zoom_level.powf(0.7) * 2.5
        }) {
            selected_star.0 = Some(index);
        } else {
            selected_star.0 = None;
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

#[derive(Component)]
struct MarkedStar;

fn display_star_selected(
    mut commands: Commands,
    selected_star: Res<SelectedStar>,
    marked: Query<Entity, With<MarkedStar>>,
    universe: Res<Universe>,
) {
    if selected_star.is_changed() {
        if let Ok(entity) = marked.get_single() {
            commands.entity(entity).despawn_recursive()
        };
        if let Some(index) = selected_star.0 {
            commands
                .entity(universe.star_entities[index])
                .with_children(|parent| {
                    let shape = shapes::RegularPolygon {
                        sides: 5,
                        feature: shapes::RegularPolygonFeature::Radius(4.0),
                        ..shapes::RegularPolygon::default()
                    };
                    parent.spawn((
                        GeometryBuilder::build_as(
                            &shape,
                            DrawMode::Stroke(StrokeMode::new(
                                Color::rgb(0.5, 1.25, 0.5),
                                0.5 / <StarSize as Into<f32>>::into(universe.galaxy[index].size),
                            )),
                            Transform::from_translation(Vec3::new(
                                0.0,
                                0.0,
                                z_levels::STAR_SELECTION,
                            )),
                        ),
                        MarkedStar,
                    ));
                });
        }
    }
}

fn rotate_mark(mut query: Query<&mut Transform, With<MarkedStar>>, time: Res<Time>) {
    let delta = time.delta_seconds();

    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_z(-0.15 * delta));
    }
}
