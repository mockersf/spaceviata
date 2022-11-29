use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef, ShaderType},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    window::WindowResized,
};
use rand::Rng;

use crate::{game::z_levels, GameState};

use super::world::{CameraController, RATIO_ZOOM_DISTANCE};

const CURRENT_STATE: GameState = GameState::Game;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(Material2dPlugin::<StarfieldMaterial>::default())
            .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            .add_system_set(SystemSet::on_update(CURRENT_STATE).with_system(update_starfield))
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down));
    }
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
struct ScreenTag;

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone, ShaderType)]
#[uuid = "D80C1B8C-4023-47E4-BFB6-29616A0DBF70"]
#[uniform(0, StarfieldMaterial)]
pub struct StarfieldMaterial {
    position: Vec2,
    seeds: Vec2,
}

impl<'a> From<&'a StarfieldMaterial> for StarfieldMaterial {
    fn from(material: &'a StarfieldMaterial) -> Self {
        Self {
            position: material.position,
            seeds: material.seeds,
        }
    }
}

impl Material2d for StarfieldMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        ShaderRef::Path("shaders/starfield.wgsl".into())
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StarfieldMaterial>>,
    windows: Res<Windows>,
) {
    info!("Loading screen");

    let window = windows.primary();
    commands.spawn(((
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
            material: materials.add(StarfieldMaterial {
                position: Vec2::ZERO,
                seeds: Vec2::new(
                    rand::thread_rng().gen_range(0.0..1000.0),
                    rand::thread_rng().gen_range(0.0..1000.0),
                ),
            }),
            transform: Transform::from_translation(Vec2::ZERO.extend(z_levels::STARFIELD))
                .with_scale(Vec2::splat(window.width().max(window.height())).extend(1.0)),
            ..default()
        },
        ScreenTag,
    ),));
}

fn update_starfield(
    controller: Res<CameraController>,
    mut starfield: Query<&mut Transform, With<Handle<StarfieldMaterial>>>,
    mut materials: ResMut<Assets<StarfieldMaterial>>,
    mut resized: EventReader<WindowResized>,
) {
    if controller.is_changed() {
        let mut starfield_transform = starfield.single_mut();
        starfield_transform.translation.x =
            controller.position.x * controller.zoom_level / RATIO_ZOOM_DISTANCE;
        starfield_transform.translation.y =
            controller.position.y * controller.zoom_level / RATIO_ZOOM_DISTANCE;

        for (_, material) in materials.iter_mut() {
            material.position = controller.position;
        }
    }
    if let Some(resized) = resized.iter().last() {
        let mut starfield_transform = starfield.single_mut();
        starfield_transform.scale.x = resized.width.max(resized.height);
        starfield_transform.scale.y = resized.width.max(resized.height);
    }
}
