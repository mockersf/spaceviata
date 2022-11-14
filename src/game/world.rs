use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{assets::GalaxyAssets, game::World, GameState};

use super::galaxy::{Star, StarColor};

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
                    .with_system(update_camera_controller),
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
    game: Res<World>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    info!("Loading screen");

    for star in &game.galaxy {
        commands.spawn((
            (
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
            ),
            System { star: star.clone() },
        ));
    }

    commands.insert_resource(CameraController {
        zoom_level: 1.0,
        position: Vec2::ZERO,
    });
    commands.insert_resource(CameraControllerTarget {
        zoom_level: 8.0,
        position: game.galaxy[game.start[0]].position,
    });
    *camera.single_mut() = Camera2dBundle::default().transform;
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
    let speed = 8.0;
    if controller.position.distance_squared(target.position) > 15.0 {
        let towards = (target.position - controller.position) * time.delta_seconds() * speed;
        controller.position += towards;
    }
    if (controller.zoom_level - target.zoom_level).abs() > 0.05 {
        let towards = (target.zoom_level - controller.zoom_level) * time.delta_seconds() * speed;
        controller.zoom_level += towards;
    }
}
