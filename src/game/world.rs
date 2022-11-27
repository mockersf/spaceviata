use std::{f32::consts::FRAC_PI_8, time::Duration};

use bevy::{
    input::{mouse::MouseWheel, touch::TouchPhase},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_easings::{EaseValue, Lerp};

use crate::{
    assets::{GalaxyAssets, UiAssets},
    game::{galaxy::StarSize, z_levels, CurrentGame, Universe},
    GameState,
};

use super::{
    fleet::Order,
    galaxy::{GalaxyCreator, Star, StarColor},
    StarState,
};

pub const RATIO_ZOOM_DISTANCE: f32 = 2.0;

const CURRENT_STATE: GameState = GameState::Game;

#[derive(Component)]
struct ScreenTag;

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            .add_system_set(
                SystemSet::on_update(CURRENT_STATE)
                    .with_system(update_camera)
                    .with_system(update_camera_controller)
                    .with_system(camera_keyboard_controls)
                    .with_system(camera_mouse_controls)
                    .with_system(camera_touch_controls)
                    .with_system(hide_stars),
            )
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down));
    }
}

#[derive(Resource)]
pub struct CameraController {
    pub zoom_level: f32,
    pub position: Vec2,
}

#[derive(Resource)]
pub struct CameraControllerTarget {
    pub zoom_level: f32,
    pub position: Vec2,
    pub ignore_movement: bool,
}

#[derive(Component)]
struct System {
    star: Star,
}

#[derive(Component)]
struct StarName;
#[derive(Component)]
struct StarHat;

#[derive(Resource)]
struct TempMaterials {
    blue_star: Handle<ColorMaterial>,
    yellow_star: Handle<ColorMaterial>,
    orange_star: Handle<ColorMaterial>,
}

fn setup(
    mut commands: Commands,
    galaxy_assets: Res<GalaxyAssets>,
    ui_assets: Res<UiAssets>,
    mut universe: ResMut<Universe>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Loading screen");

    let blue = materials.get(&galaxy_assets.blue_star).unwrap().clone();
    let blue_star = materials.add(blue);
    let yellow = materials.get(&galaxy_assets.yellow_star).unwrap().clone();
    let yellow_star = materials.add(yellow);
    let orange = materials.get(&galaxy_assets.orange_star).unwrap().clone();
    let orange_star = materials.add(orange);
    let temp_materials = TempMaterials {
        blue_star,
        yellow_star,
        orange_star,
    };

    universe.star_entities = universe
        .galaxy
        .iter()
        .zip(universe.players[0].vision.iter())
        .map(|(star, visibility)| {
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: galaxy_assets.star_mesh.clone_weak().into(),
                        material: match (star.color, visibility) {
                            (StarColor::Blue, StarState::Unknown) => {
                                temp_materials.blue_star.clone_weak()
                            }
                            (StarColor::Orange, StarState::Unknown) => {
                                temp_materials.yellow_star.clone_weak()
                            }
                            (StarColor::Yellow, StarState::Unknown) => {
                                temp_materials.orange_star.clone_weak()
                            }
                            (StarColor::Blue, _) => galaxy_assets.yellow_star.clone_weak(),
                            (StarColor::Orange, _) => galaxy_assets.yellow_star.clone_weak(),
                            (StarColor::Yellow, _) => galaxy_assets.orange_star.clone_weak(),
                        },
                        transform: Transform::from_translation(
                            star.position.extend(z_levels::STAR),
                        )
                        .with_scale(Vec3::splat(star.size.into())),
                        ..default()
                    },
                    ScreenTag,
                    System { star: star.clone() },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text2dBundle {
                            text: Text::from_section(
                                &star.name,
                                TextStyle {
                                    font: ui_assets.font_main.clone_weak(),
                                    font_size: 40.0,
                                    color: Color::WHITE,
                                },
                            ),
                            transform: Transform::from_scale(Vec3::splat(
                                0.1 / <StarSize as Into<f32>>::into(star.size),
                            ))
                            .with_translation(Vec3::new(
                                -(star.name.len() as f32) / 2.0,
                                -2.2,
                                z_levels::STAR_NAME,
                            )),
                            ..default()
                        },
                        StarName,
                    ));
                    let hat_angle = -FRAC_PI_8;
                    parent.spawn((
                        SpriteBundle {
                            texture: galaxy_assets.hat.clone_weak(),
                            transform: Transform::from_scale(Vec3::splat(0.0075))
                                .with_translation(
                                    (Vec2::new(-hat_angle.sin(), hat_angle.cos()) * 2.75)
                                        .extend(z_levels::STAR_DECORATION),
                                )
                                .with_rotation(Quat::from_rotation_z(hat_angle)),
                            visibility: Visibility {
                                is_visible: *visibility != StarState::Unknown,
                            },
                            ..default()
                        },
                        StarHat,
                    ));
                })
                .id()
        })
        .collect();

    commands.insert_resource(CameraController {
        zoom_level: 1.0,
        position: Vec2::ZERO,
    });
    commands.insert_resource(CameraControllerTarget {
        zoom_level: 8.0,
        position: universe.galaxy[universe.players[0].start].position,
        ignore_movement: false,
    });
    *camera.single_mut() = Camera2dBundle::default().transform;

    commands.insert_resource(CurrentGame {
        start: time.last_update().unwrap(),
    });

    commands.insert_resource(temp_materials);
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[allow(clippy::type_complexity)]
fn update_camera(
    controller: Res<CameraController>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut systems: Query<(&mut Transform, &System), Without<Camera2d>>,
    mut star_names: Query<&mut Visibility, With<StarName>>,
    mut fleets: Query<(&mut Transform, &Order), (Without<Camera2d>, Without<System>)>,
    universe: Res<Universe>,
) {
    if controller.is_changed() {
        let mut camera_transform = camera.single_mut();
        camera_transform.translation.x =
            controller.position.x * controller.zoom_level / RATIO_ZOOM_DISTANCE;
        camera_transform.translation.y =
            controller.position.y * controller.zoom_level / RATIO_ZOOM_DISTANCE;

        for (mut transform, system) in &mut systems {
            transform.scale = Vec3::splat(
                <StarSize as Into<f32>>::into(system.star.size) * controller.zoom_level.powf(0.7),
            );
            transform.translation = (system.star.position * controller.zoom_level
                / RATIO_ZOOM_DISTANCE)
                .extend(z_levels::STAR);
        }
        for (mut transform, order) in &mut fleets {
            transform.scale = Vec3::splat(controller.zoom_level.powf(0.7));
            let star = match order {
                Order::Orbit(around) => around,
                Order::Move { from, .. } => from,
            };
            transform.translation = (universe.galaxy[*star].position * controller.zoom_level
                / RATIO_ZOOM_DISTANCE)
                .extend(z_levels::SHIP);
        }
        if controller.zoom_level < 4.0 {
            for mut visibility in &mut star_names {
                if visibility.is_visible {
                    visibility.is_visible = false;
                }
            }
        } else {
            for mut visibility in &mut star_names {
                if !visibility.is_visible {
                    visibility.is_visible = true;
                }
            }
        }
    }
}

fn update_camera_controller(
    mut controller: ResMut<CameraController>,
    target: Res<CameraControllerTarget>,
    time: Res<Time>,
) {
    let speed = 5.0;
    if controller.position.distance_squared(target.position) > 15.0 {
        let towards = (target.position - controller.position) * time.delta_seconds() * speed;
        controller.position += towards;
    }
    if (controller.zoom_level - target.zoom_level).abs() > 0.01 {
        let towards = (target.zoom_level - controller.zoom_level) * time.delta_seconds() * speed;
        controller.zoom_level += towards;
    }
}

#[inline]
fn limit_camera_controller(new_position: Vec2, size: f32) -> bool {
    new_position.distance(Vec2::ZERO) < size * 100.0
}

fn camera_keyboard_controls(
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    keyboard_input: Res<Input<KeyCode>>,
    galaxy_settings: Res<GalaxyCreator>,
) {
    let mut order = Vec2::ZERO;
    if keyboard_input.any_pressed([KeyCode::Right, KeyCode::D]) {
        order.x += 1.0;
    }
    if keyboard_input.any_pressed([KeyCode::Left, KeyCode::A]) {
        order.x -= 1.0;
    }
    if keyboard_input.any_pressed([KeyCode::Up, KeyCode::W]) {
        order.y += 1.0;
    }
    if keyboard_input.any_pressed([KeyCode::Down, KeyCode::S]) {
        order.y -= 1.0;
    }
    if order != Vec2::ZERO {
        let order = order.normalize();
        let new_position = controller.position + order * (controller.zoom_level / 4.0 + 10.0);
        if limit_camera_controller(new_position, galaxy_settings.size) {
            target.position = new_position;
        }
    }
    if keyboard_input.just_pressed(KeyCode::PageUp) {
        target.zoom_level = (controller.zoom_level + 1.0).min(10.0);
    }
    if keyboard_input.just_pressed(KeyCode::PageDown) {
        target.zoom_level = (controller.zoom_level - 1.0).max(1.0);
    }
}

enum DragState {
    NotPressed,
    WaitForFirstDistance(Vec2),
    Dragging,
}

impl Default for DragState {
    fn default() -> Self {
        DragState::NotPressed
    }
}

const DRAG_DISTANCE: f32 = 10f32;

#[cfg(not(target_arch = "wasm32"))]
fn camera_mouse_controls(
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut drag_state: Local<DragState>,
    galaxy_settings: Res<GalaxyCreator>,
    windows: Res<Windows>,
) {
    use super::ui::LEFT_PANEL_WIDTH;

    if target.ignore_movement {
        *drag_state = DragState::NotPressed;
        return;
    }
    if windows.primary().cursor_position().is_none()
        || windows.primary().cursor_position().unwrap().x < LEFT_PANEL_WIDTH
    {
        *drag_state = DragState::NotPressed;
        return;
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        *drag_state = DragState::WaitForFirstDistance(Vec2::ZERO);
    }
    if mouse_input.just_released(MouseButton::Left) {
        *drag_state = DragState::NotPressed;
    }

    match &mut *drag_state {
        DragState::NotPressed => {}
        DragState::WaitForFirstDistance(distance) => {
            for motion in mouse_motion.iter() {
                *distance += motion.delta;
            }
            if DRAG_DISTANCE < distance.length() {
                *drag_state = DragState::Dragging;
            }
        }
        DragState::Dragging => {
            for motion in mouse_motion.iter() {
                let new_position = controller.position
                    + (motion.delta * Vec2::new(-1.0, 1.0))
                        * (RATIO_ZOOM_DISTANCE * 20.0 / controller.zoom_level);
                if limit_camera_controller(new_position, galaxy_settings.size) {
                    target.position = new_position;
                }
            }
        }
    }
    mouse_motion.clear();
    for wheel in mouse_wheel.iter() {
        target.zoom_level = (controller.zoom_level - wheel.y).clamp(1.0, 10.0);
    }
}

#[cfg(target_arch = "wasm32")]
fn camera_mouse_controls(
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    mouse_input: Res<Input<MouseButton>>,
    mut cursor_moved: EventReader<CursorMoved>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut drag_state: Local<DragState>,
    mut last_position: Local<Option<Vec2>>,
    galaxy_settings: Res<GalaxyCreator>,
) {
    if target.ignore_movement {
        *drag_state = DragState::NotPressed;
        return;
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        *drag_state = DragState::WaitForFirstDistance(Vec2::ZERO);
        *last_position = None;
    }
    if mouse_input.just_released(MouseButton::Left) {
        *drag_state = DragState::NotPressed;
    }

    match &mut *drag_state {
        DragState::NotPressed => {}
        DragState::WaitForFirstDistance(distance) => {
            for motion in cursor_moved.iter() {
                if last_position.is_none() {
                    *last_position = Some(motion.position);
                }
                *distance = motion.position - last_position.unwrap();
            }
            if DRAG_DISTANCE < distance.length() {
                *drag_state = DragState::Dragging;
            }
        }
        DragState::Dragging => {
            for cursor in cursor_moved.iter() {
                if let Some(last_position) = *last_position {
                    let new_position = controller.position
                        + (cursor.position - last_position)
                            * Vec2::new(-1.0, -1.0)
                            * (40.0 / controller.zoom_level);
                    if limit_camera_controller(new_position, galaxy_settings.size) {
                        target.position = new_position;
                    }
                }
                *last_position = Some(cursor.position);
            }
        }
    }
    cursor_moved.clear();
    for wheel in mouse_wheel.iter() {
        target.zoom_level = (controller.zoom_level - wheel.y).clamp(1.0, 10.0);
    }
}

fn camera_touch_controls(
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    mut touches: EventReader<TouchInput>,
    mut last_position: Local<Option<Vec2>>,
    mut pressed_at: Local<Option<Duration>>,
    time: Res<Time>,
) {
    if target.ignore_movement {
        *pressed_at = None;
        return;
    }

    for touch in touches.iter() {
        if touch.phase == TouchPhase::Started {
            *pressed_at = Some(time.raw_elapsed());
            *last_position = None;
        }
        if let Some(last_position) = *last_position {
            if let Some(when) = *pressed_at {
                if (time.raw_elapsed() - when).as_secs_f32() > 0.2 {
                    target.position = controller.position
                        + (touch.position - last_position)
                            * Vec2::new(-1.0, 1.0)
                            * (40.0 / controller.zoom_level);
                }
            }
        }
        *last_position = Some(touch.position);
    }
}

fn hide_stars(
    mut commands: Commands,
    mut stars: Query<&mut Handle<ColorMaterial>>,
    galaxy_assets: Res<GalaxyAssets>,
    universe: Res<Universe>,
    time: Res<Time>,
    current: ResMut<CurrentGame>,
    temp_materials: Option<Res<TempMaterials>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    if let Some(temp_materials) = temp_materials {
        let duration = 10.0;
        let spent = (time.last_update().unwrap() - current.start).as_secs_f32();
        let unknown = materials.get(&galaxy_assets.unknown).unwrap().color;

        let mut blue = materials.get_mut(&temp_materials.blue_star).unwrap();
        blue.color = EaseValue(blue.color)
            .lerp(&EaseValue(unknown), &(spent / duration))
            .0;
        let mut yellow = materials.get_mut(&temp_materials.yellow_star).unwrap();
        yellow.color = EaseValue(yellow.color)
            .lerp(&EaseValue(unknown), &(spent / duration))
            .0;
        let mut orange = materials.get_mut(&temp_materials.orange_star).unwrap();
        orange.color = EaseValue(orange.color)
            .lerp(&EaseValue(unknown), &(spent / duration))
            .0;
        if spent > duration {
            commands.remove_resource::<TempMaterials>();
            for (entity, visible) in universe
                .star_entities
                .iter()
                .zip(universe.players[0].vision.iter())
            {
                if *visible == StarState::Unknown {
                    *stars.get_mut(*entity).unwrap() = galaxy_assets.unknown.clone_weak();
                }
            }
        }
    }
}
