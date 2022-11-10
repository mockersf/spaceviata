use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{assets::GalaxyAssets, game::World, GameState};

use super::galaxy::StarColor;

const CURRENT_STATE: GameState = GameState::Game;

#[derive(Component)]
struct ScreenTag;

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            // .add_system_set(SystemSet::on_update(CURRENT_STATE))
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down));
    }
}

fn setup(mut commands: Commands, galaxy_assets: Res<GalaxyAssets>, game: Res<World>) {
    info!("Loading screen");

    for star in &game.galaxy {
        commands.spawn((
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
        ));
    }

    for (i, start) in game.start.iter().enumerate() {
        let star = game.galaxy.get(*start).unwrap();
        commands.spawn(SpriteBundle {
            transform: Transform::from_translation(star.position.extend(0.2))
                .with_scale(Vec3::splat(star.size.into()) * 15.0),
            sprite: Sprite {
                color: Color::rgba(
                    if i == 0 { 0.0 } else { 1.0 },
                    if i == 0 { 1.0 } else { 0.0 },
                    0.0,
                    0.3,
                ),
                ..default()
            },
            ..default()
        });
    }
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
