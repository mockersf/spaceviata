use bevy::prelude::*;

use rand::seq::SliceRandom;

use crate::{assets::UiAssets, ui_helper::ColorScheme};

const CURRENT_STATE: crate::GameState = crate::GameState::Game;

#[derive(Component)]
struct ScreenTag;
#[derive(Component)]
struct RootEvents;

#[derive(Resource)]
struct EventsPossible {
    // for inspiration: https://www.gq.com/story/the-34-most-egregious-sci-fi-movie-cliches-in-interstellar
    events: Vec<&'static str>,
}

#[derive(Resource)]
struct EventsSpawn {
    /// How often to spawn a new event?
    timer: Timer,
}

#[derive(Component)]
struct DestroyAfter {
    /// How often to spawn a new event?
    timer: Timer,
}

impl Default for EventsPossible {
    fn default() -> Self {
        EventsPossible {
            events: vec![
                "A Shmorr went to the sky and shtroumpfed a reactor, he is now lost forever...",
                "A singularity appeared somewhere...",
                "Bad aliens invaded a planet!",
                "Good aliens saved a planet!",
                "We conquered our planet back, Hail Life!",
                "Our planet has fallen, Despair...",
                "'An asteroïd might destroy all life!' says reknown whistleblower.",
                "An AI took control of a spaceship fleet, only YOU can stop it!",
                "After 10 000 years lost in space, Granger Taylor returns to Earth.",
                "Frederick Valentich found in Vosl'tir",
                "Space dogs won Galaxy Soccer Cup, with more than 33 balls played!",
                "All robot meat of age are called to register before the 44th of year Agick'ti",
                "Please do not click so fast.",
            ],
        }
    }
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EventsPossible::default())
            .insert_resource(EventsSpawn {
                timer: Timer::from_seconds(5f32, TimerMode::Repeating),
            })
            .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
            .add_system_set(
                SystemSet::on_update(CURRENT_STATE)
                    .with_system(create_events)
                    .with_system(destroy_after),
            );
    }
}

fn setup(mut commands: Commands) {
    let root_width = Val::Percent(15f32);
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: root_width,
                    left: Val::Undefined,
                    bottom: Val::Undefined,
                    top: Val::Percent(20.),
                },
                size: Size {
                    width: root_width,
                    height: Val::Auto,
                },
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        },
        ScreenTag,
        RootEvents,
    ));
}

#[derive(Component)]
struct PlayerName;

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn create_events(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    windows: Res<Windows>,
    time: Res<Time>,
    events: Res<EventsPossible>,
    mut spawn_timer: ResMut<EventsSpawn>,
    root_query: Query<Entity, With<RootEvents>>,
) {
    spawn_timer.timer.tick(time.delta());
    if spawn_timer.timer.finished() {
        let font_details = ui_handles.font_sub.clone_weak();
        let width = windows.primary().width();

        let event = events
            .events
            .choose(&mut rand::thread_rng())
            .unwrap_or(&"An unknown event occured.");

        let event_toast = {
            commands
                .spawn((
                    TextBundle {
                        style: Style {
                            size: Size {
                                width: Val::Px(width / 4.0),
                                ..Default::default()
                            },
                            position: UiRect {
                                right: Val::Px(-width / 50.0),
                                ..default()
                            },
                            ..Default::default()
                        },
                        text: Text::from_section(
                            event.to_string(),
                            TextStyle {
                                font: font_details,
                                color: ColorScheme::TEXT,
                                font_size: width / 40.0,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    },
                    DestroyAfter {
                        timer: Timer::from_seconds(5f32, TimerMode::Once),
                    },
                ))
                .id()
        };
        commands.entity(root_query.single()).add_child(event_toast);
    }
}

/// This could use a fixed time (f32 or duration, to avoid mutability in loop)
fn destroy_after(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DestroyAfter)>,
) {
    for (e, mut d) in query.iter_mut() {
        d.timer.tick(time.delta());
        if d.timer.just_finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}
