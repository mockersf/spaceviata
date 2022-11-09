use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    sprite::MaterialMesh2dBundle,
};

use crate::{
    assets::{GalaxyAssets, UiAssets},
    game::galaxy::GalaxyKind,
    ui_helper::{button::ButtonId, ColorScheme},
    GameState,
};

use super::galaxy::{GalaxyCreator, StarColor};

const CURRENT_STATE: crate::GameState = crate::GameState::Setup;

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
                    .with_system(setting_button)
                    .with_system(action_button),
            )
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down));
    }
}

#[derive(Component)]
struct GalaxyPreview;

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    windows: Res<Windows>,
    mut images: ResMut<Assets<Image>>,
) {
    info!("Loading screen");

    let size = Extent3d {
        width: 1024,
        height: 1024,
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };
    image.resize(size);

    let image_handle = images.add(image);

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                priority: -1,
                target: RenderTarget::Image(image_handle.clone()),

                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
        UiCameraConfig {
            show_ui: false,
            ..default()
        },
    ));
    let galaxy = GalaxyCreator {
        generated: Vec::new(),
        nb_players: 2,
        size: SizeControl::default().into(),
        density: DensityControl::default().into(),
        _kind: GalaxyKind::default(),
    };

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
                custom_size: Some(Vec2::splat(1024.0)),
                color: Color::rgb(0.01, 0.01, 0.15),
                ..default()
            },
            ..default()
        },
        RenderLayers::layer(1),
        GalaxyPreview,
        ScreenTag,
    ));

    let base = commands
        .spawn((
            NodeBundle {
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
            },
            ScreenTag,
        ))
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
        let mut children = vec![text];
        for size_control in [SizeControl::Small, SizeControl::Medium, SizeControl::Large] {
            let button_entity = button.add(
                &mut commands,
                Val::Px(height / 6.0),
                Val::Px(height / 20.0),
                UiRect::all(Val::Auto),
                ui_handles.font_main.clone_weak(),
                GalaxyControl::Size(size_control),
                height / 40.0,
            );
            if size_control == SizeControl::default() {
                commands.entity(button_entity).insert(Selected);
            }
            children.push(button_entity);
        }
        commands.entity(row).push_children(&children);
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
        let mut children = vec![text];
        for density_control in [
            DensityControl::Sparse,
            DensityControl::Normal,
            DensityControl::Dense,
        ] {
            let button_entity = button.add(
                &mut commands,
                Val::Px(height / 6.0),
                Val::Px(height / 20.0),
                UiRect::all(Val::Auto),
                ui_handles.font_main.clone_weak(),
                GalaxyControl::Density(density_control),
                height / 40.0,
            );
            if density_control == DensityControl::default() {
                commands.entity(button_entity).insert(Selected);
            }
            children.push(button_entity);
        }
        commands.entity(row).push_children(&children);
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
        let mut children = vec![text];
        for nb in [2, 3, 4, 5] {
            let button_entity = button.add(
                &mut commands,
                Val::Px(height / 8.0),
                Val::Px(height / 20.0),
                UiRect::all(Val::Auto),
                ui_handles.font_main.clone_weak(),
                GalaxyControl::Players(nb),
                height / 40.0,
            );
            if nb == galaxy.nb_players {
                commands.entity(button_entity).insert(Selected);
            }
            children.push(button_entity);
        }
        commands.entity(row).push_children(&children);
        row
    };

    let action_buttons = {
        let row = commands
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    margin: UiRect {
                        top: Val::Px(height / 15.0),
                        ..default()
                    },
                    size: Size {
                        width: Val::Percent(100.0),
                        height: Val::Undefined,
                    },
                    justify_content: JustifyContent::SpaceEvenly,
                    ..default()
                },
                ..Default::default()
            })
            .id();
        let cancel = button.add(
            &mut commands,
            Val::Px(height / 5.0),
            Val::Px(height / 15.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            Action::Cancel,
            height / 30.0,
        );
        let start = button.add(
            &mut commands,
            Val::Px(height / 5.0),
            Val::Px(height / 15.0),
            UiRect::all(Val::Auto),
            ui_handles.font_main.clone_weak(),
            Action::Start,
            height / 30.0,
        );
        commands.entity(row).push_children(&[cancel, start]);
        row
    };

    commands.entity(base).push_children(&[
        row_type,
        row_size,
        row_density,
        row_players,
        action_buttons,
    ]);

    commands.spawn((
        ImageBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(0.),
                    ..default()
                },
                size: Size::new(Val::Percent(50.0), Val::Undefined),

                ..default()
            },
            image: UiImage(image_handle),
            ..default()
        },
        ScreenTag,
    ));
    commands.insert_resource(galaxy);
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
                GalaxyControl::Size(size) => creator.size = size.into(),
                GalaxyControl::Density(density) => creator.density = density.into(),
                GalaxyControl::Players(nb) => creator.nb_players = nb,
                GalaxyControl::Kind(_) => (),
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Action {
    Start,
    Cancel,
}

impl From<Action> for String {
    fn from(action: Action) -> Self {
        match action {
            Action::Start => String::from("Start"),
            Action::Cancel => String::from("Cancel"),
        }
    }
}

fn action_button(
    interaction_query: Query<
        (&Interaction, &ButtonId<Action>),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<GameState>>,
) {
    for (interaction, control) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match control.0 {
                Action::Cancel => state.set(GameState::Menu).unwrap(),
                Action::Start => (),
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DensityControl {
    Sparse,
    #[default]
    Normal,
    Dense,
}

impl From<DensityControl> for u32 {
    fn from(density: DensityControl) -> Self {
        match density {
            DensityControl::Sparse => 1,
            DensityControl::Normal => 3,
            DensityControl::Dense => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SizeControl {
    Small,
    #[default]
    Medium,
    Large,
}

impl From<SizeControl> for u32 {
    fn from(density: SizeControl) -> Self {
        match density {
            SizeControl::Small => 2,
            SizeControl::Medium => 3,
            SizeControl::Large => 5,
        }
    }
}

#[derive(Component, Clone, Copy, Debug)]
enum GalaxyControl {
    Size(SizeControl),
    Density(DensityControl),
    Players(u32),
    Kind(GalaxyKind),
}

#[allow(clippy::from_over_into)]
impl Into<String> for GalaxyControl {
    fn into(self) -> String {
        match self {
            GalaxyControl::Size(SizeControl::Small) => "small".to_string(),
            GalaxyControl::Size(SizeControl::Medium) => "medium".to_string(),
            GalaxyControl::Size(SizeControl::Large) => "large".to_string(),
            GalaxyControl::Density(DensityControl::Sparse) => "sparse".to_string(),
            GalaxyControl::Density(DensityControl::Normal) => "normal".to_string(),
            GalaxyControl::Density(DensityControl::Dense) => "dense".to_string(),
            GalaxyControl::Players(n) => format!("{}", n),
            GalaxyControl::Kind(GalaxyKind::Spiral) => "spiral".to_string(),
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
                p.spawn((
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
                    RenderLayers::layer(1),
                ));
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
