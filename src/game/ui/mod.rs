use std::{f32::consts::PI, time::Duration};

use bevy::{input::mouse::MouseMotion, math::Vec3Swizzles, prelude::*};
use bevy_easings::{Ease, EaseFunction, EasingType};
use bevy_prototype_lyon::{
    prelude::{DrawMode, GeometryBuilder, PathBuilder, StrokeMode},
    shapes,
};

use crate::{
    assets::{loader::ShipAssets, UiAssets},
    ui_helper::button::ButtonId,
    GameState,
};

use super::{
    fleet::{turns_between, FleetSize, Order, Owner, Ship, ShipKind},
    galaxy::StarSize,
    turns::TurnState,
    world::{CameraController, CameraControllerTarget, RATIO_ZOOM_DISTANCE},
    z_levels, StarState, Universe,
};

mod left_panel;
mod menu;
mod shipyard;
mod turn;

pub const LEFT_PANEL_WIDTH: f32 = 200.0;

const DAMPENER: Color = Color::rgba(0.15, 0.15, 0.15, 0.75);

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SelectedStar>()
            .init_resource::<turn::DisplayedMessage>()
            .init_resource::<shipyard::ShipyadForStar>()
            .add_event::<shipyard::ShipyardEvent>()
            .add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(button_system.before(select_star))
                    .with_system(select_star.after(button_system))
                    .with_system(left_panel::display_star_list)
                    .with_system(left_panel::star_list_click)
                    .with_system(left_panel::star_list_scroll)
                    .with_system(display_star_selected.before(dragging_ship))
                    .with_system(star_button_system)
                    .with_system(rotate_mark)
                    .with_system(dragging_ship.after(display_star_selected))
                    .with_system(left_panel::update_player_stats)
                    .with_system(turn::display_messages)
                    .with_system(shipyard::display_shipyard)
                    .with_system(shipyard::button_system)
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiButtons {
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

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
) {
    info!("loading UI");

    menu::setup(&mut commands, &ui_handles, &buttons);

    turn::setup(&mut commands, &ui_handles, &buttons);

    left_panel::setup(&mut commands, &ui_handles);

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

    // shipyard
    {
        let button_handle = ui_handles.button_handle.clone_weak();
        let button = buttons.get(&button_handle).unwrap();

        let shipyard = button.add_hidden_section(
            &mut commands,
            Val::Px(110.),
            Val::Px(40.),
            UiRect::all(Val::Auto),
            vec![
                TextSection {
                    value: material_icons::icon_to_char(material_icons::Icon::RocketLaunch)
                        .to_string(),
                    style: TextStyle {
                        font: ui_handles.font_material.clone_weak(),
                        font_size: 15.0,
                        color: crate::ui_helper::ColorScheme::TEXT,
                        ..Default::default()
                    },
                },
                TextSection {
                    value: " shipyard".to_string(),
                    style: TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        font_size: 20.0,
                        color: crate::ui_helper::ColorScheme::TEXT,
                        ..Default::default()
                    },
                },
            ],
            StarAction::Shipyard(usize::MAX),
            20.,
            false,
        );

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        display: Display::None,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            left: Val::Undefined,
                            right: Val::Undefined,
                            bottom: Val::Undefined,
                            top: Val::Undefined,
                        },
                        ..default()
                    },
                    ..default()
                },
                ShipyardButton,
                ScreenTag,
            ))
            .push_children(&[shipyard]);
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
#[derive(Component)]
struct ShipyardButton;

#[derive(Resource, Default)]
pub struct SelectedStar {
    ignore_next_click: bool,
    index: Option<usize>,
    dragging_ship: (Option<Entity>, Option<Entity>),
}

fn button_system(
    interaction_query: Query<(&Interaction, &ButtonId<UiButtons>, Changed<Interaction>)>,
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    mut state: ResMut<State<GameState>>,
    mut turn_state: ResMut<State<TurnState>>,
    mut menu_container: Query<&mut Visibility, With<menu::MenuContainer>>,
    mut displayed_message: ResMut<turn::DisplayedMessage>,
    mut selected_star: ResMut<SelectedStar>,
    mut shipyard: EventWriter<shipyard::ShipyardEvent>,
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
                (UiButtons::EndTurn, true) => {
                    turn_state.set(TurnState::Bots).unwrap();
                    shipyard.send(shipyard::ShipyardEvent::Close);
                }
                (UiButtons::NextMessage, true) | (UiButtons::LastMessage, true) => {
                    displayed_message.0 += 1;
                    selected_star.bypass_change_detection().ignore_next_click = true;
                    shipyard.send(shipyard::ShipyardEvent::Close);
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
            if selected_star.ignore_next_click {
                selected_star.bypass_change_detection().ignore_next_click = false;
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

#[allow(clippy::type_complexity)]
#[derive(Component)]
struct MarkedStar;

#[derive(Clone, Copy)]
enum StarAction {
    Ship(Entity),
    Shipyard(usize),
}

impl From<StarAction> for String {
    fn from(action: StarAction) -> Self {
        match action {
            StarAction::Ship(_) => "".to_string(),
            StarAction::Shipyard(_) => {
                material_icons::icon_to_char(material_icons::Icon::RocketLaunch).to_string()
            }
        }
    }
}

fn star_button_system(
    interaction_query: Query<(&Interaction, &ButtonId<StarAction>, Changed<Interaction>)>,
    mut target: ResMut<CameraControllerTarget>,
    mut selected_star: ResMut<SelectedStar>,
    mut shipyard: EventWriter<shipyard::ShipyardEvent>,
) {
    for (interaction, button_id, changed) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match (&button_id.0, changed) {
                (StarAction::Ship(entity), true) => {
                    target.ignore_movement = true;
                    selected_star.dragging_ship.0 = Some(*entity);
                }
                (StarAction::Shipyard(index), true) => {
                    shipyard.send(shipyard::ShipyardEvent::OpenForStar(*index));
                }
                _ => (),
            }
        }
        if *interaction == Interaction::None && changed && selected_star.index.is_some() {
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
    mut shipyard_button: Query<
        (&mut Style, &mut BackgroundColor),
        (
            With<ShipyardButton>,
            Without<StarPanel>,
            Without<FleetsPanel>,
        ),
    >,
    mut star_actions: Query<&mut ButtonId<StarAction>>,
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

            // hide shipyard panel
            let mut style = shipyard_button.single_mut().0;
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

            // hide shipyard panel
            let mut style = shipyard_button.single_mut().0;
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
                    match universe.players[0].vision[index] {
                        StarState::Owned(0) => {
                            let star_revenue = universe.star_revenue(index);
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
                                                Color::rgb(0.64, 0.17, 0.17)
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
                        StarState::Uninhabited => {
                            parent.spawn(TextBundle {
                                text: Text::from_section(
                                    "Uninhabited",
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
                                    let transform_eased =
                                        Transform::from_rotation(Quat::from_rotation_z(PI));

                                    parent.spawn((
                                        ImageBundle {
                                            image: UiImage(match ship.kind {
                                                ShipKind::Colony => {
                                                    ship_assets.colony_ship.clone_weak()
                                                }
                                                ShipKind::Fighter => {
                                                    ship_assets.fighter.clone_weak()
                                                }
                                            }),
                                            style: Style {
                                                size: Size::new(Val::Px(15.0), Val::Px(15.0)),
                                                ..default()
                                            },
                                            transform: transform_eased,
                                            ..default()
                                        },
                                        Ease::ease_to(
                                            transform_eased,
                                            Transform::from_rotation(Quat::from_rotation_z(PI))
                                                .with_scale(Vec3::ONE * 1.2f32),
                                            EaseFunction::SineInOut,
                                            EasingType::PingPong {
                                                duration: Duration::from_millis(800),
                                                pause: None,
                                            },
                                        ),
                                        Interaction::None,
                                        ButtonId(StarAction::Ship(*entity)),
                                    ));
                                    parent.spawn((
                                        TextBundle {
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
                                        },
                                        Interaction::None,
                                        ButtonId(StarAction::Ship(*entity)),
                                    ));
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
            if universe.star_details[index].owner == 0 {
                let mut style = shipyard_button.single_mut().0;
                style.display = Display::Flex;
                style.size = Size::new(Val::Px(110.0), Val::Px(40.0));
                style.position.left = Val::Px(pos.x - 55.0);
                style.position.bottom = Val::Px(pos.y + 65.0);
                for mut action in &mut star_actions {
                    if let StarAction::Shipyard(shipyard_index) = &mut action.as_mut().0 {
                        *shipyard_index = index;
                    }
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

#[derive(Component)]
pub struct OneFrameDelay;

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
                        ShipKind::Fighter => ship_assets.fighter.clone_weak(),
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
