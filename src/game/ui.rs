use std::f32::consts::PI;

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    math::Vec3Swizzles,
    prelude::*,
    ui::FocusPolicy,
};
use bevy_prototype_lyon::{
    prelude::{DrawMode, GeometryBuilder, PathBuilder, StrokeMode},
    shapes,
};

use crate::{
    assets::{loader::ShipAssets, UiAssets},
    ui_helper::button::{ButtonId, ButtonText},
    GameState,
};

use super::{
    fleet::{turns_between, FleetSize, Order, Owner, Ship, ShipKind},
    galaxy::StarSize,
    turns::{TurnState, Turns},
    world::{CameraController, CameraControllerTarget, RATIO_ZOOM_DISTANCE},
    z_levels, StarState, Universe,
};

pub const LEFT_PANEL_WIDTH: f32 = 200.0;

const DAMPENER: Color = Color::rgba(0.15, 0.15, 0.15, 0.75);

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SelectedStar>()
            .init_resource::<DisplayedMessage>()
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(button_system)
                    .with_system(select_star)
                    .with_system(display_star_list)
                    .with_system(star_list_click)
                    .with_system(star_list_scroll)
                    .with_system(display_star_selected.before(dragging_ship))
                    .with_system(star_button_system)
                    .with_system(rotate_mark)
                    .with_system(dragging_ship.after(display_star_selected))
                    .with_system(update_player_stats)
                    .with_system(display_messages)
                    .with_system(make_it_visible),
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
    EndTurn,
    NextMessage,
    LastMessage,
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
            UiButtons::EndTurn => {
                material_icons::icon_to_char(material_icons::Icon::FastForward).to_string()
            }
            UiButtons::NextMessage => {
                material_icons::icon_to_char(material_icons::Icon::NavigateNext).to_string()
            }
            UiButtons::LastMessage => {
                material_icons::icon_to_char(material_icons::Icon::Done).to_string()
            }
        }
    }
}

#[derive(Component)]
struct PlayerStatsMarker;

#[derive(Component)]
struct MessagePanelMarker;

#[derive(Component)]
struct MessageContentMarker;

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
) {
    info!("loading UI");

    // menu
    {
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
                },
                ScreenTag,
            ))
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
    }

    // turns
    {
        let button_handle = ui_handles.button_handle.clone_weak();
        let button = buttons.get(&button_handle).unwrap();
        let material = ui_handles.font_material.clone_weak();

        let end_turn = button.add(
            &mut commands,
            Val::Px(40.),
            Val::Px(40.),
            UiRect::all(Val::Auto),
            material,
            UiButtons::EndTurn,
            25.,
            crate::ui_helper::ColorScheme::TEXT,
        );

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position: UiRect {
                            right: Val::Px(20.0),
                            bottom: Val::Px(20.0),
                            ..default()
                        },
                        size: Size {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                        },
                        flex_direction: FlexDirection::Column,
                        position_type: PositionType::Absolute,
                        align_items: AlignItems::Baseline,
                        ..default()
                    },
                    ..default()
                },
                ScreenTag,
            ))
            .push_children(&[end_turn]);
    }

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

    // left panel
    {
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

    // star panel
    {
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
                        ..Default::default()
                    },
                    ..Default::default()
                },
                StarDetails,
            ))
            .id();

        let panel_style = Style {
            display: Display::None,
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Undefined,
                right: Val::Undefined,
                bottom: Val::Undefined,
                top: Val::Undefined,
            },
            margin: UiRect::all(Val::Px(0.)),
            size: Size::new(Val::Px(0.0), Val::Px(0.0)),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        };

        commands.spawn((
            bevy_ninepatch::NinePatchBundle {
                style: panel_style,
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    ui_handles.br_panel_handle.1.clone_weak(),
                    ui_handles.br_panel_handle.0.clone_weak(),
                    base,
                ),
                ..default()
            },
            StarPanel,
            ScreenTag,
        ));
    }

    // fleets panel
    {
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
                        ..Default::default()
                    },
                    ..Default::default()
                },
                FleetsDetails,
            ))
            .id();

        let panel_style = Style {
            display: Display::None,
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Undefined,
                right: Val::Undefined,
                bottom: Val::Undefined,
                top: Val::Undefined,
            },
            margin: UiRect::all(Val::Px(0.)),
            size: Size::new(Val::Px(0.0), Val::Px(0.0)),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        };

        commands.spawn((
            bevy_ninepatch::NinePatchBundle {
                style: panel_style,
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    ui_handles.tl_panel_handle.1.clone_weak(),
                    ui_handles.tl_panel_handle.0.clone_weak(),
                    base,
                ),
                ..default()
            },
            FleetsPanel,
            ScreenTag,
        ));
    }
}

#[derive(Component)]
struct StarPanel;
#[derive(Component)]
struct StarDetails;

#[derive(Component)]
struct FleetsPanel;
#[derive(Component)]
struct FleetsDetails;

#[derive(Component, Default)]
struct StarList {
    position: f32,
}

#[derive(Resource, Default)]
struct SelectedStar {
    index: Option<usize>,
    dragging_ship: (Option<Entity>, Option<Entity>),
}

#[derive(Resource, Default)]
struct DisplayedMessage(usize);

#[derive(Component)]
struct MenuContainer;

fn button_system(
    interaction_query: Query<(&Interaction, &ButtonId<UiButtons>, Changed<Interaction>)>,
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    mut state: ResMut<State<GameState>>,
    mut turn_state: ResMut<State<TurnState>>,
    mut menu_container: Query<&mut Visibility, With<MenuContainer>>,
    mut displayed_message: ResMut<DisplayedMessage>,
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
                (UiButtons::EndTurn, true) => turn_state.set(TurnState::Bots).unwrap(),
                (UiButtons::NextMessage, true) | (UiButtons::LastMessage, true) => {
                    displayed_message.0 += 1;
                }
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
    mut last_pressed: Local<f32>,
    mut pressed_at: Local<Option<Vec2>>,
    time: Res<Time>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        *last_pressed = time.elapsed_seconds();
        *pressed_at = windows.primary().cursor_position();
    }

    if selected_star.dragging_ship.0.is_none() {
        if let Some(position) = mouse_input
            .just_released(MouseButton::Left)
            .then(|| windows.primary().cursor_position().or(*pressed_at))
            .flatten()
            .or_else(|| {
                touches.first_pressed_position().map(|mut pos| {
                    pos.y = windows.primary().height() - pos.y;
                    pos
                })
            })
        {
            if position.x < LEFT_PANEL_WIDTH || time.elapsed_seconds() - *last_pressed > 0.5 {
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
                    < <StarSize as Into<f32>>::into(star.size)
                        * controller.zoom_level.powf(0.7)
                        * 2.5
            }) {
                if selected_star.index != Some(index) {
                    selected_star.index = Some(index);
                }
            } else {
                selected_star.index = None;
            }
        }
    }
}

#[derive(Component)]
struct StarListIndex(usize);

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

#[allow(clippy::type_complexity)]
fn star_list_click(
    interaction_query: Query<(&Interaction, &StarListIndex), (Changed<Interaction>, With<Button>)>,
    mut selected_star: ResMut<SelectedStar>,
    universe: Res<Universe>,
    mut controller_target: ResMut<CameraControllerTarget>,
) {
    for (interaction, star_index) in &interaction_query {
        if *interaction == Interaction::Clicked {
            if selected_star.index == Some(star_index.0) {
                controller_target.zoom_level = 8.0;
                controller_target.position = universe.galaxy[star_index.0].position;
            } else {
                selected_star.index = Some(star_index.0);
            }
        }
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
struct MarkedStar;

enum StarAction {
    Ship(Entity),
}

impl From<StarAction> for String {
    fn from(action: StarAction) -> Self {
        match action {
            StarAction::Ship(_) => "".to_string(),
        }
    }
}

fn star_button_system(
    interaction_query: Query<(&Interaction, &ButtonId<StarAction>, Changed<Interaction>)>,
    mut target: ResMut<CameraControllerTarget>,
    mut selected_star: ResMut<SelectedStar>,
) {
    for (interaction, button_id, changed) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match (&button_id.0, changed) {
                (StarAction::Ship(entity), true) => {
                    target.ignore_movement = true;
                    selected_star.dragging_ship.0 = Some(*entity);
                }
                _ => (),
            }
        }
        if *interaction == Interaction::None && changed {
            target.ignore_movement = false;
        }
    }
}

#[allow(clippy::type_complexity)]
fn display_star_selected(
    mut commands: Commands,
    selected_star: Res<SelectedStar>,
    marked: Query<Entity, With<MarkedStar>>,
    universe: Res<Universe>,
    mut star_panel: Query<(&mut Style, &mut BackgroundColor), With<StarPanel>>,
    star_details: Query<Entity, With<StarDetails>>,
    mut fleets_panel: Query<
        (&mut Style, &mut BackgroundColor),
        (With<FleetsPanel>, Without<StarPanel>),
    >,
    fleets_details: Query<Entity, With<FleetsDetails>>,
    transform: Query<&GlobalTransform>,
    camera: Query<(&GlobalTransform, &Camera, Changed<GlobalTransform>)>,
    ui_assets: Res<UiAssets>,
    camera_controller: Res<CameraController>,
    fleets: Query<(Entity, &Ship, &Order, &FleetSize, &Owner)>,
    ship_assets: Res<ShipAssets>,
) {
    if selected_star.is_changed() {
        if selected_star.dragging_ship.0.is_some() {
            // hide star panel
            let mut style = star_panel.single_mut().0;
            style.display = Display::None;
            style.size = Size::new(Val::Px(0.0), Val::Px(0.0));

            // hide fleets panel
            let mut style = fleets_panel.single_mut().0;
            style.display = Display::None;
            style.size = Size::new(Val::Px(0.0), Val::Px(0.0));

            return;
        }

        if let Ok(entity) = marked.get_single() {
            // remove star selection mark
            commands.entity(entity).despawn_recursive();

            // hide star panel
            let mut style = star_panel.single_mut().0;
            style.display = Display::None;
            style.size = Size::new(Val::Px(0.0), Val::Px(0.0));

            // hide fleets panel
            let mut style = fleets_panel.single_mut().0;
            style.display = Display::None;
            style.size = Size::new(Val::Px(0.0), Val::Px(0.0));
        };
    }

    if let Some(index) = selected_star.index {
        let star = &universe.galaxy[index];
        if selected_star.is_changed() {
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
                                #[cfg(target_arch = "wasm32")]
                                Color::rgb(0.5, 1.0, 0.5),
                                #[cfg(not(target_arch = "wasm32"))]
                                Color::rgb(0.5, 1.25, 0.5),
                                0.5 / <StarSize as Into<f32>>::into(star.size),
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
            {
                let details_entity = star_details.single();
                commands.entity(details_entity).despawn_descendants();
                commands.entity(details_entity).with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            star.name.clone(),
                            TextStyle {
                                font: ui_assets.font_main.clone_weak(),
                                font_size: 20.0,
                                color: Color::WHITE,
                            },
                        ),
                        style: Style {
                            size: Size {
                                width: Val::Undefined,
                                height: Val::Px(25.0),
                            },
                            margin: UiRect::top(Val::Px(-10.0)),
                            ..default()
                        },
                        ..default()
                    });
                    let star_revenue = universe.star_revenue(index);
                    match universe.players[0].vision[index] {
                        StarState::Owned(0) => {
                            parent.spawn(TextBundle {
                                text: Text::from_sections([
                                    TextSection {
                                        value: "Owned by you\n".to_string(),
                                        style: TextStyle {
                                            font: ui_assets.font_sub.clone_weak(),
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                        },
                                    },
                                    TextSection {
                                        value: format!(
                                            "Population {:.1}\n",
                                            universe.star_details[index].population
                                        ),
                                        style: TextStyle {
                                            font: ui_assets.font_sub.clone_weak(),
                                            font_size: 20.0,
                                            color: Color::WHITE,
                                        },
                                    },
                                    TextSection {
                                        value: format!("Revenue    {:.1}\n", star_revenue),
                                        style: TextStyle {
                                            font: ui_assets.font_sub.clone_weak(),
                                            font_size: 20.0,
                                            color: if star_revenue < 0.0 {
                                                Color::RED
                                            } else {
                                                Color::GREEN
                                            },
                                        },
                                    },
                                    TextSection {
                                        value: format!(
                                            "Resources  {:.1}\n",
                                            universe.star_ressource(index)
                                        ),
                                        style: TextStyle {
                                            font: ui_assets.font_sub.clone_weak(),
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
                            });
                        }
                        StarState::Owned(i) => {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    format!("Last seen: Player {}", i),
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
                        StarState::Unknown => {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "Unknown",
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
                    }
                });
            }
            {
                let details_entity = fleets_details.single();
                commands.entity(details_entity).despawn_descendants();

                let fleets = fleets
                    .iter()
                    .filter(|(_, _, order, _, owner)| {
                        if owner.0 == 0 {
                            match order {
                                Order::Orbit(around) => *around == index,
                                Order::Move { from, to: _, step } => *from == index && *step == 0,
                            }
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<_>>();
                if !fleets.is_empty() {
                    commands.entity(details_entity).with_children(|parent| {
                        for (entity, ship, order, fleet_size, _) in &fleets {
                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        ..default()
                                    },
                                    ..default()
                                })
                                .with_children(|parent| {
                                    parent.spawn((
                                        ImageBundle {
                                            image: UiImage(match ship.kind {
                                                ShipKind::Colony => {
                                                    ship_assets.colony_ship.clone_weak()
                                                }
                                            }),
                                            style: Style {
                                                size: Size::new(Val::Px(15.0), Val::Px(15.0)),
                                                ..default()
                                            },
                                            transform: Transform::from_rotation(
                                                Quat::from_rotation_z(PI),
                                            ),
                                            ..default()
                                        },
                                        Interaction::None,
                                        ButtonId(StarAction::Ship(*entity)),
                                    ));
                                    parent.spawn(TextBundle {
                                        text: Text::from_sections([
                                            TextSection {
                                                value: " ".to_string(),
                                                style: TextStyle {
                                                    font: ui_assets.font_sub.clone_weak(),
                                                    font_size: 20.0,
                                                    color: Color::WHITE,
                                                },
                                            },
                                            TextSection {
                                                value: match order {
                                                    Order::Move { .. } => {
                                                        material_icons::icon_to_char(
                                                            material_icons::Icon::Start,
                                                        )
                                                        .to_string()
                                                    }
                                                    Order::Orbit(_) => {
                                                        material_icons::icon_to_char(
                                                            material_icons::Icon::Refresh,
                                                        )
                                                        .to_string()
                                                    }
                                                },
                                                style: TextStyle {
                                                    font: ui_assets.font_material.clone_weak(),
                                                    font_size: 15.0,
                                                    color: match order {
                                                        Order::Move { .. } => Color::GREEN,
                                                        Order::Orbit(_) => Color::WHITE,
                                                    },
                                                },
                                            },
                                            TextSection {
                                                value: format!(" {} {}\n", fleet_size, ship),
                                                style: TextStyle {
                                                    font: ui_assets.font_sub.clone_weak(),
                                                    font_size: 20.0,
                                                    color: Color::WHITE,
                                                },
                                            },
                                        ]),
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
                                });
                        }
                    });
                }
            }
        }

        let (camera_transform, camera, changed_transform) = camera.single();
        if selected_star.is_changed() || changed_transform {
            let transform = transform.get(universe.star_entities[index]).unwrap();
            let pos = camera
                .world_to_viewport(camera_transform, transform.translation())
                .unwrap();
            {
                let (mut style, mut background_color) = star_panel.single_mut();
                background_color.0 = DAMPENER;
                style.display = Display::Flex;
                style.size = Size::new(Val::Px(200.0), Val::Px(120.0));
                style.position.left = Val::Px(
                    pos.x
                        + <StarSize as Into<f32>>::into(star.size)
                            * 5.0
                            * camera_controller.zoom_level.powf(0.7),
                );
                let Val::Px(height) = style.size.height else{
                    return;
                };
                style.position.bottom = Val::Px(pos.y - height / 2.0);
            }
            {
                let has_fleets = fleets.iter().any(|(_, _, order, _, owner)| {
                    if owner.0 == 0 {
                        match order {
                            Order::Orbit(around) => *around == index,
                            Order::Move { from, to: _, step } => *from == index && *step == 0,
                        }
                    } else {
                        false
                    }
                });
                if has_fleets {
                    let (mut style, mut background_color) = fleets_panel.single_mut();
                    background_color.0 = DAMPENER;
                    style.display = Display::Flex;
                    style.size = Size::new(Val::Px(200.0), Val::Px(120.0));
                    style.position.left = Val::Px(
                        pos.x
                            - <StarSize as Into<f32>>::into(star.size)
                                * 5.0
                                * camera_controller.zoom_level.powf(0.7)
                            - 200.0,
                    );
                    let Val::Px(height) = style.size.height else{
                    return;
                };
                    style.position.bottom = Val::Px(pos.y - height / 2.0);
                }
            }
        }
    }
}

fn rotate_mark(mut query: Query<&mut Transform, With<MarkedStar>>, time: Res<Time>) {
    let delta = time.delta_seconds();

    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_z(-0.15 * delta));
    }
}

fn update_player_stats(
    mut text: Query<&mut Text, With<PlayerStatsMarker>>,
    universe: Res<Universe>,
) {
    if universe.is_changed() {
        let mut text = text.single_mut();
        text.sections[1].value = format!("{:.1}\n", universe.player_population(0));
        let revenue = universe.player_revenue(0);
        text.sections[3].value = format!("{:.1}\n", universe.player_revenue(0));
        if revenue < 0.0 {
            text.sections[3].style.color = Color::RED
        } else {
            text.sections[3].style.color = Color::GREEN
        }
        text.sections[5].value = format!("{:.1}\n", universe.players[0].savings);
        if universe.players[0].savings < 0.0 {
            text.sections[5].style.color = Color::RED
        } else {
            text.sections[5].style.color = Color::GREEN
        }
        text.sections[7].value = format!("{:.1}\n", universe.players[0].resources);
    }
}

fn display_messages(
    mut commands: Commands,
    turns: Res<Turns>,
    mut current_message: ResMut<DisplayedMessage>,
    ui_handles: Res<UiAssets>,
    panel: Query<Entity, With<MessagePanelMarker>>,
    mut content: Query<&mut Text, With<MessageContentMarker>>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    mut text: Query<(&mut Text, &ButtonText<UiButtons>), Without<MessageContentMarker>>,
) {
    if turns.is_changed() && !turns.messages.is_empty() {
        if let Ok(entity) = panel.get_single() {
            commands.entity(entity).despawn_recursive();
        };

        current_message.0 = 0;

        let button_handle = ui_handles.button_handle.clone_weak();
        let button = buttons.get(&button_handle).unwrap();

        let next_message_button = button.add_hidden(
            &mut commands,
            Val::Px(30.),
            Val::Px(30.),
            UiRect::all(Val::Auto),
            ui_handles.font_material.clone_weak(),
            if turns.messages.len() == 1 {
                UiButtons::LastMessage
            } else {
                UiButtons::NextMessage
            },
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
                parent.spawn((
                    TextBundle {
                        text: Text::from_sections(
                            turns.messages[current_message.0].as_sections(&ui_handles),
                        ),
                        style: Style {
                            size: Size {
                                width: Val::Undefined,
                                height: Val::Px(100.0),
                            },
                            ..default()
                        },
                        ..default()
                    },
                    MessageContentMarker,
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
                    .push_children(&[next_message_button]);
            })
            .id();

        let panel_style = Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Px(300.0), Val::Px(200.0)),
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
                        position: UiRect {
                            right: Val::Px(0.0),
                            ..default()
                        },
                        position_type: PositionType::Absolute,
                        size: Size::new(Val::Px(400.0), Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                },
                MessagePanelMarker,
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
    if current_message.is_changed() {
        if current_message.0 == turns.messages.len() {
            let Ok(entity) = panel.get_single() else {
                return
            };
            commands.entity(entity).despawn_recursive();
        } else if current_message.0 > 0 {
            if current_message.0 == turns.messages.len() - 1 {
                for (mut text, button) in &mut text {
                    if button.0 == UiButtons::NextMessage {
                        text.sections[0].value =
                            material_icons::icon_to_char(material_icons::Icon::Done).to_string();
                    }
                }
            }
            content.single_mut().sections =
                turns.messages[current_message.0].as_sections(&ui_handles);
        }
    }
}

#[derive(Component)]
pub(crate) struct OneFrameDelay;

fn make_it_visible(
    mut commands: Commands,
    mut style: Query<(Entity, &mut Style), With<OneFrameDelay>>,
) {
    for (entity, mut style) in &mut style {
        commands.entity(entity).remove::<OneFrameDelay>();
        style.display = Display::Flex;
    }
}

#[allow(clippy::type_complexity)]
fn dragging_ship(
    mut commands: Commands,
    mut selected_star: ResMut<SelectedStar>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    universe: Res<Universe>,
    controller: Res<CameraController>,
    ship_assets: Res<ShipAssets>,
    ui_assets: Res<UiAssets>,
    camera: Query<(&Camera, &GlobalTransform)>,
    windows: Res<Windows>,
    mut transform: Query<&mut Transform>,
    time: Res<Time>,
    fleets: Query<&Ship>,
    mut over_star: Local<Option<(usize, [Entity; 2], Entity, usize)>>,
) {
    if selected_star.is_changed() {
        if let (Some(fleet_entity), None) = selected_star.dragging_ship {
            let fleet = fleets.get(fleet_entity).unwrap();
            let position = windows
                .primary()
                .cursor_position()
                .and_then(|cursor| {
                    let (camera, transform) = camera.single();
                    camera.viewport_to_world(transform, cursor)
                })
                .map(|ray| ray.origin)
                .unwrap_or_default()
                .xy()
                .extend(z_levels::SHIP_DRAGGING);
            let ship_entity = commands
                .spawn(SpriteBundle {
                    texture: match fleet.kind {
                        ShipKind::Colony => ship_assets.colony_ship.clone_weak(),
                    },
                    transform: Transform::from_translation(position)
                        .with_scale(Vec3::splat(0.2))
                        .with_rotation(Quat::from_rotation_z(PI)),
                    ..default()
                })
                .id();
            selected_star.dragging_ship.1 = Some(ship_entity);
        }
    }
    if selected_star.dragging_ship.0.is_some() {
        if mouse_input.just_released(MouseButton::Left) {
            if over_star.is_none() {
                // reset order to orbiting
                commands
                    .entity(selected_star.dragging_ship.0.unwrap())
                    .insert(Order::Orbit(selected_star.index.unwrap()));
                selected_star.set_changed();
            }
            if let Some(entity) = selected_star.dragging_ship.1 {
                commands.entity(entity).despawn_recursive();
                selected_star.dragging_ship.0 = None;
                selected_star.dragging_ship.1 = None;
            }
            return;
        }
        if let Some(dragged_ship_entity) = selected_star.dragging_ship.1 {
            if let Ok(mut transform) = transform.get_mut(dragged_ship_entity) {
                transform.rotation =
                    Quat::from_rotation_z(PI + (time.elapsed_seconds() * 10.0).sin() / 2.0);
                for motion in mouse_motion.iter() {
                    #[cfg(target_arch = "wasm32")]
                    let fixer = Vec2::new(1.0, -1.0) / windows.primary().scale_factor() as f32;
                    #[cfg(not(target_arch = "wasm32"))]
                    let fixer = Vec2::new(1.0, -1.0);
                    transform.translation = (transform.translation.xy() + motion.delta * fixer)
                        .extend(z_levels::SHIP_DRAGGING);
                }
                let hover = transform.translation.xy();
                if let Some((index, _)) = universe.galaxy.iter().enumerate().find(|(_, star)| {
                    (star.position * controller.zoom_level / RATIO_ZOOM_DISTANCE).distance(hover)
                        < <StarSize as Into<f32>>::into(star.size)
                            * controller.zoom_level.powf(0.7)
                            * 2.5
                }) {
                    if over_star.is_none() {
                        let mut path_builder = PathBuilder::new();
                        let from = universe.galaxy[selected_star.index.unwrap()].position;
                        let to = universe.galaxy[index].position;
                        let turns = turns_between(from, to);
                        let length = commands
                            .spawn(Text2dBundle {
                                text: Text::from_section(
                                    format!("{} turn(s)", turns),
                                    TextStyle {
                                        font: ui_assets.font_main.clone_weak(),
                                        font_size: 25.0,
                                        color: Color::ANTIQUE_WHITE,
                                    },
                                ),
                                transform: Transform::from_translation(
                                    (((to - from) / 2.0 + from) * controller.zoom_level
                                        / RATIO_ZOOM_DISTANCE)
                                        .extend(z_levels::SHIP_DRAGGING),
                                ),
                                ..default()
                            })
                            .id();
                        path_builder.move_to(from * controller.zoom_level / RATIO_ZOOM_DISTANCE);
                        path_builder.line_to(to * controller.zoom_level / RATIO_ZOOM_DISTANCE);
                        let line = path_builder.build();
                        let path = commands
                            .spawn(GeometryBuilder::build_as(
                                &line,
                                DrawMode::Stroke(StrokeMode::new(
                                    Color::rgb(0.75, 0.75, 0.75),
                                    1.5,
                                )),
                                Transform::from_translation(
                                    Vec2::ZERO.extend(z_levels::STAR_SELECTION),
                                ),
                            ))
                            .id();
                        *over_star = Some((
                            index,
                            [path, length],
                            selected_star.dragging_ship.0.unwrap(),
                            selected_star.index.unwrap(),
                        ));
                    }
                } else if let Some((_, entities, _, _)) = *over_star {
                    commands.entity(entities[0]).despawn_recursive();
                    commands.entity(entities[1]).despawn_recursive();
                    *over_star = None;
                }
            }
        }
    } else if let Some((index, entities, fleet_entity, from_star)) = *over_star {
        commands.entity(entities[0]).despawn_recursive();
        commands.entity(entities[1]).despawn_recursive();
        *over_star = None;
        commands.entity(fleet_entity).insert(Order::Move {
            from: from_star,
            to: index,
            step: 0,
        });
        selected_star.set_changed();
    }
    mouse_motion.clear();
}
