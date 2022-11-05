use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

use crate::assets::UiAssets;

const CURRENT_STATE: crate::GameState = crate::GameState::Playing;

#[derive(Component)]
struct ScreenTag;

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down));
    }
}

struct GalaxyCreator {
    stars: u32,
    arms: u32,
    radius: f32,
    generated: Vec<Vec2>,
}

impl Iterator for GalaxyCreator {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stars == 0 {
            return None;
        }

        let mut rand = rand::thread_rng();
        let arm_angle = ((360 / self.arms) % 360) as f32;
        let angular_spread = 180 / (self.arms * 2);

        self.stars -= 1;

        'distance: loop {
            let distance_to_center = rand.gen_range(0.05..=1.0_f32).sqrt() * self.radius;
            let angle = rand.gen_range(0.0..(angular_spread as f32));
            // * rand.gen_bool(0.5).then_some(1.0).unwrap_or(-1.0);

            let spiral_angle = 0.75;

            let arm = (rand.gen::<u32>() % self.arms) as f32 * arm_angle;

            let x = distance_to_center
                * (PI / 180.0 * (arm + distance_to_center * spiral_angle + angle) as f32).cos();
            let y = distance_to_center
                * (PI / 180.0 * (arm + distance_to_center * spiral_angle + angle) as f32).sin();
            let new_star = Vec2::new(x, y);

            for other_star in &self.generated {
                if new_star.distance(*other_star) < self.radius / (7.0 * self.arms as f32) {
                    continue 'distance;
                }
            }
            self.generated.push(new_star);
            return Some(new_star);
        }
    }
}

fn setup(
    mut commands: Commands,
    _ui_handles: Res<UiAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Loading screen");

    let nb_arms = 2;

    let galaxy = GalaxyCreator {
        stars: 50 * nb_arms,
        arms: nb_arms,
        radius: 350.0,
        generated: Vec::new(),
    };
    let mesh = meshes.add(shape::Circle::new(2.5).into());
    let material = materials.add(ColorMaterial::from(Color::PURPLE));
    commands.spawn(MaterialMesh2dBundle {
        mesh: mesh.clone().into(),
        material: material.clone(),
        transform: Transform::from_translation(Vec3::ZERO),
        ..default()
    });

    for star in galaxy {
        commands.spawn(MaterialMesh2dBundle {
            mesh: mesh.clone_weak().into(),
            material: material.clone_weak(),
            transform: Transform::from_translation(star.extend(0.0)),
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
