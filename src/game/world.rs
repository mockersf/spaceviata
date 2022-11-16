use std::time::Duration;

use bevy::{
    input::{mouse::MouseWheel, touch::TouchPhase},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};

use crate::{
    assets::GalaxyAssets,
    game::{CurrentGame, World},
    GameState,
};

use super::{
    galaxy::{Star, StarColor},
    StarState,
};

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
struct CameraController {
    zoom_level: f32,
    position: Vec2,
}

#[derive(Resource)]
struct CameraControllerTarget {
    zoom_level: f32,
    position: Vec2,
}

#[derive(Component)]
struct System {
    star: Star,
}

fn setup(
    mut commands: Commands,
    galaxy_assets: Res<GalaxyAssets>,
    mut world: ResMut<World>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    time: Res<Time>,
) {
    info!("Loading screen");

    // for star in &game.galaxy {
    world.star_entities = world
        .galaxy
        .iter()
        .map(|star| {
            commands
                .spawn((
                    MaterialMesh2dBundle {
                        mesh: galaxy_assets.star_mesh.clone_weak().into(),
                        material: match star.color {
                            StarColor::Blue => galaxy_assets.blue_star.clone_weak(),
                            StarColor::Yellow => galaxy_assets.yellow_star.clone_weak(),
                            StarColor::Orange => galaxy_assets.orange_star.clone_weak(),
                        },
                        transform: Transform::from_translation(star.position.extend(0.1))
                            .with_scale(Vec3::splat(star.size.into())),
                        ..default()
                    },
                    ScreenTag,
                    System { star: star.clone() },
                ))
                .id()
        })
        .collect();

    commands.insert_resource(CameraController {
        zoom_level: 1.0,
        position: Vec2::ZERO,
    });
    commands.insert_resource(CameraControllerTarget {
        zoom_level: 8.0,
        position: world.galaxy[world.players[0].start].position,
    });
    *camera.single_mut() = Camera2dBundle::default().transform;

    commands.insert_resource(CurrentGame {
        start: time.last_update().unwrap(),
        init: false,
    });
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_camera(
    controller: Res<CameraController>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
    mut systems: Query<(&mut Transform, &System), Without<Camera2d>>,
) {
    if controller.is_changed() {
        let mut camera_transform = camera.single_mut();
        camera_transform.translation.x = controller.position.x * controller.zoom_level / 2.0;
        camera_transform.translation.y = controller.position.y * controller.zoom_level / 2.0;

        for (mut transform, system) in &mut systems {
            transform.scale = Vec3::splat(system.star.size.into()) * controller.zoom_level;
            transform.translation =
                (system.star.position * controller.zoom_level / 2.0).extend(0.1);
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

fn camera_keyboard_controls(
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    keyboard_input: Res<Input<KeyCode>>,
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
        target.position = controller.position + order * (controller.zoom_level / 4.0 + 10.0);
    }
    if keyboard_input.just_pressed(KeyCode::PageUp) {
        target.zoom_level = (controller.zoom_level + 1.0).min(10.0);
    }
    if keyboard_input.just_pressed(KeyCode::PageDown) {
        target.zoom_level = (controller.zoom_level - 1.0).max(1.0);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn camera_mouse_controls(
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut pressed_at: Local<Option<Duration>>,
    time: Res<Time>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        *pressed_at = Some(time.raw_elapsed())
    }
    if mouse_input.just_released(MouseButton::Left) {
        *pressed_at = None;
    }
    if let Some(when) = *pressed_at {
        if (time.raw_elapsed() - when).as_secs_f32() > 0.2 {
            for motion in mouse_motion.iter() {
                target.position = controller.position
                    + (motion.delta * Vec2::new(-1.0, 1.0)) * (40.0 / controller.zoom_level);
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
    mut pressed_at: Local<Option<Duration>>,
    mut last_position: Local<Option<Vec2>>,
    time: Res<Time>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        *pressed_at = Some(time.raw_elapsed());
        *last_position = None;
    }
    if mouse_input.just_released(MouseButton::Left) {
        *pressed_at = None;
    }
    if let Some(when) = *pressed_at {
        if (time.raw_elapsed() - when).as_secs_f32() > 0.2 {
            for cursor in cursor_moved.iter() {
                if let Some(last_position) = *last_position {
                    target.position = controller.position
                        + (cursor.position - last_position)
                            * Vec2::new(-1.0, -1.0)
                            * (40.0 / controller.zoom_level);
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
    mut stars: Query<&mut Handle<ColorMaterial>>,
    galaxy_assets: Res<GalaxyAssets>,
    world: Res<World>,
    time: Res<Time>,
    mut current: ResMut<CurrentGame>,
) {
    if (time.last_update().unwrap() - current.start).as_secs_f32() > 0.5 && !current.init {
        for (entity, visible) in world
            .star_entities
            .iter()
            .zip(world.players[0].vision.iter())
        {
            if *visible == StarState::Unknown {
                *stars.get_mut(*entity).unwrap() = galaxy_assets.unknown.clone_weak();
            }
        }
        current.init = true;
    }
}
