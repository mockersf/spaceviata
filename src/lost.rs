use bevy::prelude::*;

use crate::assets::UiAssets;

const CURRENT_STATE: crate::GameState = crate::GameState::Lost;

#[derive(Component)]
struct ScreenTag;

#[derive(Resource)]
struct Screen {
    done: Timer,
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Screen {
            done: Timer::from_seconds(20.0, TimerMode::Once),
        })
        .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
        .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
        .add_system_set(SystemSet::on_update(CURRENT_STATE).with_system(done));
    }
}

fn setup(mut commands: Commands, _ui_handles: Res<UiAssets>) {
    info!("Loading screen");

    commands.insert_resource(Screen {
        done: Timer::from_seconds(20.0, TimerMode::Once),
    });
}

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn done(time: Res<Time>, mut screen: ResMut<Screen>, mut state: ResMut<State<crate::GameState>>) {
    if screen.done.tick(time.delta()).finished() {
        state.set(crate::GameState::Menu).unwrap();
    }
}
