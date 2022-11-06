use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

use crate::{
    assets::{GalaxyAssets, UiAssets},
    ui_helper::{button::ButtonId, ColorScheme},
};

const CURRENT_STATE: crate::GameState = crate::GameState::Playing;

#[derive(Component)]
struct ScreenTag;

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            .add_system_set(
                SystemSet::on_update(CURRENT_STATE)
                    .with_system(display_galaxy)
                    .with_system(ui_button_system)
                    .with_system(setting_button),
            )
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down));
    }
}

#[derive(Clone, Copy, Debug)]
enum GalaxyKind {
    Spiral,
}

#[derive(Resource)]
struct GalaxyCreator {
    nb_players: u32,
    size: u32,
    density: u32,
    _kind: GalaxyKind,
    generated: Vec<Vec2>,
}

impl Iterator for GalaxyCreator {
    type Item = Vec2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.generated.len() as u32 == self.nb_players * self.size * self.density * 3 {
            return None;
        }

        let mut rand = rand::thread_rng();
        let arm_angle = ((360 / self.nb_players) % 360) as f32;
        let angular_spread = 180 / (self.nb_players * 2);

        let mut fail = 0;

        'distance: loop {
            let distance_to_center =
                rand.gen_range(0.03..=1.0_f32).sqrt() * self.size as f32 * 100.0;
            let angle = rand.gen_range(0.0..(angular_spread as f32));
            // * rand.gen_bool(0.5).then_some(1.0).unwrap_or(-1.0);

            let spiral_angle = 0.75;

            let arm = (rand.gen::<u32>() % self.nb_players) as f32 * arm_angle;

            let x = distance_to_center
                * (PI / 180.0 * (arm + distance_to_center * spiral_angle + angle) as f32).cos();
            let y = distance_to_center
                * (PI / 180.0 * (arm + distance_to_center * spiral_angle + angle) as f32).sin();
            let new_star = Vec2::new(x, y);

            for other_star in &self.generated {
                let distance = new_star.distance(*other_star);
                if distance < 100.0 / (self.density as f32) {
                    fail += 1;
                    if fail < self.generated.len() || distance < 100.0 / (self.density as f32 * 2.0)
                    {
                        continue 'distance;
                    }
                }
            }
            self.generated.push(new_star);
            return Some(new_star);
        }
    }
}

#[derive(Component)]
struct GalaxyPreview;

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    windows: Res<Windows>,
) {
    info!("Loading screen");

    let galaxy = GalaxyCreator {
        generated: Vec::new(),
        nb_players: 2,
        size: 3,
        density: 5,
        _kind: GalaxyKind::Spiral,
    };
    commands.insert_resource(galaxy);

    let category_style = Style {
        size: Size {
            height: Val::Px(25.0),
            width: Val::Px(100.0),
            ..Default::default()
        },
        margin: UiRect {
            top: Val::Px(10.0),
            ..default()
        },
        ..Default::default()
    };
    let row_style = Style {
        flex_direction: FlexDirection::Row,
        margin: UiRect::all(Val::Px(10.0)),
        size: Size {
            width: Val::Percent(100.0),
            height: Val::Undefined,
        },
        justify_content: JustifyContent::SpaceBetween,
        ..default()
    };

    let button_handle = ui_handles.button_handle.clone_weak();
    let button = buttons.get(&button_handle).unwrap();

    let window = windows.primary();
    let height = window.height();
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(1000.0)),
                color: Color::MIDNIGHT_BLUE,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(
                window.width() / 2.0 - 250.0,
                height / 2.0 - 250.0,
                0.0,
            ))
            .with_scale(Vec3::splat(0.5)),
            ..default()
        },
        GalaxyPreview,
    ));

    let base = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                margin: UiRect::all(Val::Px(10.0)),
                size: Size {
                    width: Val::Percent(100.0),
                    height: Val::Undefined,
                },
                ..Default::default()
            },
            ..Default::default()
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
        size: Size::new(Val::Percent(50.0), Val::Percent(100.0)),
        align_content: AlignContent::Stretch,
        flex_direction: FlexDirection::Column,
        ..Default::default()
    };

    commands.spawn((
        bevy_ninepatch::NinePatchBundle {
            style: panel_style,
            nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                ui_handles.left_panel_handle.1.clone_weak(),
                ui_handles.left_panel_handle.0.clone_weak(),
                base,
            ),
            ..Default::default()
        },
        ScreenTag,
    ));

    let row_type = {
        let row = commands
            .spawn(NodeBundle {
                style: row_style.clone(),
                ..Default::default()
            })
            .id();
        let text = commands
            .spawn(TextBundle {
                style: category_style.clone(),
                text: Text::from_section(
                    "type".to_string(),
                    TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        color: ColorScheme::TEXT,
                        font_size: height / 30.0,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .id();
        let spiral = button.add(
            &mut commands,
            Val::Px(height / 6.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Kind(GalaxyKind::Spiral),
            height / 40.0,
        );
        commands.entity(spiral).insert(Selected);
        commands.entity(row).push_children(&[text, spiral]);
        row
    };

    let row_size = {
        let row = commands
            .spawn(NodeBundle {
                style: row_style.clone(),
                ..Default::default()
            })
            .id();
        let text = commands
            .spawn(TextBundle {
                style: category_style.clone(),
                text: Text::from_section(
                    "size".to_string(),
                    TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        color: ColorScheme::TEXT,
                        font_size: height / 30.0,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .id();
        let small = button.add(
            &mut commands,
            Val::Px(height / 6.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Size(2),
            height / 40.0,
        );
        let medium = button.add(
            &mut commands,
            Val::Px(height / 6.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Size(3),
            height / 40.0,
        );
        let large = button.add(
            &mut commands,
            Val::Px(height / 6.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Size(5),
            height / 40.0,
        );
        commands.entity(medium).insert(Selected);
        commands
            .entity(row)
            .push_children(&[text, small, medium, large]);
        row
    };

    let row_density = {
        let row = commands
            .spawn(NodeBundle {
                style: row_style.clone(),
                ..Default::default()
            })
            .id();
        let text = commands
            .spawn(TextBundle {
                style: category_style.clone(),
                text: Text::from_section(
                    "density".to_string(),
                    TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        color: ColorScheme::TEXT,
                        font_size: height / 30.0,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .id();
        let sparse = button.add(
            &mut commands,
            Val::Px(height / 6.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Density(3),
            height / 40.0,
        );
        let normal = button.add(
            &mut commands,
            Val::Px(height / 6.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Density(5),
            height / 40.0,
        );
        let dense = button.add(
            &mut commands,
            Val::Px(height / 6.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Density(7),
            height / 40.0,
        );
        commands.entity(normal).insert(Selected);
        commands
            .entity(row)
            .push_children(&[text, sparse, normal, dense]);
        row
    };

    let row_players = {
        let row = commands
            .spawn(NodeBundle {
                style: row_style,
                ..Default::default()
            })
            .id();
        let text = commands
            .spawn(TextBundle {
                style: category_style,
                text: Text::from_section(
                    "players".to_string(),
                    TextStyle {
                        font: ui_handles.font_main.clone_weak(),
                        color: ColorScheme::TEXT,
                        font_size: height / 30.0,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .id();
        let nb_2 = button.add(
            &mut commands,
            Val::Px(height / 10.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Players(2),
            height / 40.0,
        );
        let nb_3 = button.add(
            &mut commands,
            Val::Px(height / 10.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Players(3),
            height / 40.0,
        );
        let nb_4 = button.add(
            &mut commands,
            Val::Px(height / 10.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Players(4),
            height / 40.0,
        );
        let nb_5 = button.add(
            &mut commands,
            Val::Px(height / 10.0),
            Val::Px(height / 20.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            GalaxyControl::Players(5),
            height / 40.0,
        );

        commands.entity(nb_2).insert(Selected);
        commands
            .entity(row)
            .push_children(&[text, nb_2, nb_3, nb_4, nb_5]);
        row
    };

    commands
        .entity(base)
        .push_children(&[row_type, row_size, row_density, row_players]);
}

const SELECTED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const SELECTED_HOVERED_BUTTON: Color = Color::rgb(0.45, 0.85, 0.45);

#[allow(clippy::type_complexity)]
fn ui_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&Selected>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in &mut interaction_query {
        *color = match (*interaction, selected) {
            (Interaction::Clicked, _) => SELECTED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => SELECTED_HOVERED_BUTTON.into(),
            (Interaction::None, Some(_)) => SELECTED_BUTTON.into(),
            (Interaction::None, None) => Color::NONE.into(),
            _ => *color,
        }
    }
}

#[allow(clippy::type_complexity)]
fn setting_button(
    interaction_query: Query<
        (&Interaction, &ButtonId<GalaxyControl>, Entity, &Parent),
        (Changed<Interaction>, With<Button>),
    >,
    mut selected_query: Query<(Entity, &mut BackgroundColor, &Parent), With<Selected>>,
    mut commands: Commands,
    mut creator: ResMut<GalaxyCreator>,
) {
    for (interaction, control, entity, clicked_parent) in &interaction_query {
        if *interaction == Interaction::Clicked {
            let (previous_button, mut previous_color, _parent) = selected_query
                .iter_mut()
                .find(|(_, _, parent)| *parent == clicked_parent)
                .unwrap();
            *previous_color = Color::NONE.into();
            commands.entity(previous_button).remove::<Selected>();
            commands.entity(entity).insert(Selected);
            match control.0 {
                GalaxyControl::Size(size) => creator.size = size,
                GalaxyControl::Density(density) => creator.density = density,
                GalaxyControl::Players(nb) => creator.nb_players = nb,
                GalaxyControl::Kind(_) => (),
            }
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
enum GalaxyControl {
    Size(u32),
    Density(u32),
    Players(u32),
    Kind(GalaxyKind),
}

#[allow(clippy::from_over_into)]
impl Into<String> for GalaxyControl {
    fn into(self) -> String {
        match self {
            GalaxyControl::Size(2) => "small".to_string(),
            GalaxyControl::Size(3) => "medium".to_string(),
            GalaxyControl::Size(5) => "large".to_string(),
            GalaxyControl::Density(3) => "sparse".to_string(),
            GalaxyControl::Density(5) => "normal".to_string(),
            GalaxyControl::Density(7) => "dense".to_string(),
            GalaxyControl::Players(n) => format!("{}", n),
            GalaxyControl::Kind(GalaxyKind::Spiral) => "spiral".to_string(),
            _ => unreachable!(),
        }
    }
}

#[derive(Component)]
struct Selected;

fn display_galaxy(
    mut commands: Commands,
    mut creator: ResMut<GalaxyCreator>,
    galaxy_assets: Res<GalaxyAssets>,
    preview: Query<Entity, With<GalaxyPreview>>,
) {
    if creator.is_changed() {
        creator.generated = Vec::new();
        let entity = preview.single();
        commands.entity(entity).despawn_descendants();
        commands.entity(entity).with_children(|p| {
            for star in creator.into_iter() {
                p.spawn(MaterialMesh2dBundle {
                    mesh: galaxy_assets.star_mesh.clone_weak().into(),
                    material: galaxy_assets.star_material.clone_weak(),
                    transform: Transform::from_translation(star.extend(0.1))
                        .with_scale(Vec3::splat(1.5)),
                    ..default()
                });
            }
        });
    }
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
