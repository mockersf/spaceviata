use bevy::prelude::*;
use rand::Rng;

use crate::{assets::AllTheLoading, ui_helper::ColorScheme};

const CURRENT_STATE: crate::GameState = crate::GameState::Splash;

#[derive(Component)]
struct ScreenTag;

#[derive(Resource)]
struct Screen {
    done: Timer,
}

pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Screen {
            done: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
        .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
        .add_system_set(
            SystemSet::on_update(CURRENT_STATE)
                .with_system(done)
                .with_system(animate_logo),
        );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading screen");

    let vleue_logo = asset_server.load("branding/logo.png");
    let bevy_logo = asset_server.load("branding/bevy_logo_dark.png");
    let birdoggo_logo = asset_server.load("branding/birdoggo.png");

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            commands.spawn((
                ImageBundle {
                    style: Style {
                        size: Size::new(Val::Px(150.0), Val::Auto),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },

                    image: UiImage(vleue_logo),
                    ..default()
                },
                SplashGiggle(Timer::from_seconds(0.05, TimerMode::Repeating)),
            ));
            commands.spawn(TextBundle {
                style: Style {
                    position: UiRect {
                        left: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    ..default()
                },
                text: Text::from_section(
                    "Loading Assets...",
                    TextStyle {
                        font: asset_server.load("fonts/mandrill.ttf"),
                        font_size: 20.0,
                        color: ColorScheme::TEXT_DARK,
                    },
                ),
                ..default()
            });
            commands.spawn(ImageBundle {
                style: Style {
                    position: UiRect {
                        right: Val::Px(10.0),
                        bottom: Val::Px(10.0),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    ..default()
                },
                image: UiImage(bevy_logo),
                ..default()
            });
            commands.spawn(ImageBundle {
                style: Style {
                    position: UiRect {
                        right: Val::Px(10.0),
                        bottom: Val::Px(70.0),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    size: Size::new(Val::Auto, Val::Px(50.0)),
                    ..default()
                },
                image: UiImage(birdoggo_logo),
                ..default()
            });
        })
        .insert(ScreenTag);
}

#[derive(Component)]
struct SplashGiggle(Timer);

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn done(
    time: Res<Time>,
    mut screen: ResMut<Screen>,
    mut state: ResMut<State<crate::GameState>>,
    loading_state: Res<State<AllTheLoading>>,
) {
    if screen.done.tick(time.delta()).finished() && loading_state.current() == &AllTheLoading::Done
    {
        state.set(crate::GameState::Menu).unwrap();
    }
}

fn animate_logo(time: Res<Time>, mut query: Query<(&mut SplashGiggle, &mut Transform)>) {
    for (mut timer, mut transform) in query.iter_mut() {
        if timer.0.tick(time.delta()).just_finished() {
            let scale = transform.scale;
            if (scale.x - 1.) > 0.01 {
                *transform = Transform::IDENTITY;
                continue;
            }

            let mut rng = rand::thread_rng();
            let act = rng.gen_range(0..100);
            if act > 50 {
                let scale_diff = 0.02;
                let new_scale: f32 = rng.gen_range((1. - scale_diff)..(1. + scale_diff));
                *transform = Transform::from_scale(Vec3::splat(new_scale));
            }
        }
    }
}
