use std::{f32::consts::PI, fmt};

use bevy::prelude::*;

use crate::{assets::loader::ShipAssets, GameState};

use super::{
    world::{CameraController, RATIO_ZOOM_DISTANCE},
    z_levels, FleetsToSpawn, Universe,
};

#[derive(Component)]
pub enum Order {
    Orbit(usize),
    Move { from: usize, to: usize, step: u32 },
}

pub enum ShipKind {
    Colony,
}

#[derive(Component)]
pub struct Ship {
    pub kind: ShipKind,
}

#[derive(Component)]
pub struct FleetSize(pub u32);

#[derive(Component)]
pub struct Owner(pub usize);

#[derive(Bundle)]
pub struct Fleet {
    pub order: Order,
    pub ship: Ship,
    pub size: FleetSize,
    pub owner: Owner,
}

impl fmt::Display for FleetSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for Ship {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl fmt::Display for ShipKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ShipKind::Colony => "Colony Ship",
            }
        )
    }
}

const CURRENT_STATE: GameState = GameState::Game;

#[derive(Component)]
struct ScreenTag;

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(CURRENT_STATE)
                .with_system(spawn_fleets)
                .with_system(place_fleets),
        )
        .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down));
    }
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_fleets(
    mut commands: Commands,
    universe: Res<Universe>,
    mut fleets: ResMut<FleetsToSpawn>,
    ship_assets: Res<ShipAssets>,
    camera_controller: Res<CameraController>,
) {
    for fleet in fleets.0.drain(..) {
        let Order::Orbit(around) = fleet.order else {
            continue;
        };
        let Owner(owner) = fleet.owner;
        let mut builder = commands.spawn((
            fleet,
            SpatialBundle::from_transform(Transform::from_translation(
                (universe.galaxy[around].position * camera_controller.zoom_level
                    / RATIO_ZOOM_DISTANCE)
                    .extend(z_levels::SHIP),
            )),
            ScreenTag,
        ));
        if owner == 0 {
            builder.with_children(|parent| {
                parent.spawn(SpriteBundle {
                    transform: Transform::from_scale(Vec3::splat(0.02)),
                    texture: ship_assets.colony_ship.clone_weak(),
                    ..default()
                });
            });
        }
    }
}

#[derive(Component)]
struct Orbiting {
    since: f32,
    size: f32,
}

#[derive(Component)]
struct MovingTo {
    from: Vec2,
    to: Vec2,
    step: u32,
    size: f32,
}

#[allow(clippy::type_complexity)]
fn place_fleets(
    mut commands: Commands,
    fleets: Query<(Entity, &Order, &Children), Changed<Order>>,
    mut fleets_position: ParamSet<(
        Query<(&mut Transform, &Orbiting)>,
        Query<(&mut Transform, &MovingTo)>,
    )>,
    time: Res<Time>,
    universe: Res<Universe>,
    camera_controller: Res<CameraController>,
) {
    for (entity, order, children) in &fleets {
        match order {
            Order::Orbit(around) => {
                let star_size = universe.galaxy[*around].size;
                commands.entity(entity).insert(
                    Transform::from_translation(
                        (universe.galaxy[*around].position * camera_controller.zoom_level
                            / RATIO_ZOOM_DISTANCE)
                            .extend(z_levels::SHIP),
                    )
                    .with_scale(Vec3::splat(camera_controller.zoom_level.powf(0.7))),
                );
                commands
                    .entity(children[0])
                    .remove::<MovingTo>()
                    .insert(Orbiting {
                        since: time.elapsed_seconds(),
                        size: star_size.into(),
                    });
            }
            Order::Move { from, to, step } => {
                commands
                    .entity(children[0])
                    .remove::<Orbiting>()
                    .insert(MovingTo {
                        from: universe.galaxy[*from].position,
                        to: universe.galaxy[*to].position,
                        step: *step,
                        size: universe.galaxy[*from].size.into(),
                    });
            }
        }
    }

    for (mut transform, orbiting) in &mut fleets_position.p0() {
        transform.translation = Vec3::new(
            (time.elapsed_seconds() - orbiting.since).cos() * orbiting.size * 4.0,
            (time.elapsed_seconds() - orbiting.since).sin() * orbiting.size * 4.0,
            z_levels::SHIP,
        );
        transform.rotation = Quat::from_rotation_z(time.elapsed_seconds() - orbiting.since + PI)
    }
    for (mut transform, moving_to) in &mut fleets_position.p1() {
        let direction = moving_to.to - moving_to.from;
        let steps = turns_between(moving_to.from, moving_to.to) as f32;
        transform.translation = ((direction * moving_to.step as f32 / steps)
            + (direction.normalize() * moving_to.size * 4.0))
            .extend(z_levels::SHIP);
        transform.rotation = Quat::from_rotation_z(-direction.angle_between(Vec2::Y) + PI);
    }
}

pub fn turns_between(from: Vec2, to: Vec2) -> u32 {
    (from.distance(to) / 100.0).exp().floor().max(1.0) as u32
}
