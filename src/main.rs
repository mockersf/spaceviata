// disable console opening on windows
#![windows_subsystem = "windows"]
#![allow(clippy::needless_update, clippy::too_many_arguments)]

use bevy::{app::AppExit, prelude::*};

mod assets;
// mod game;
mod lost;
mod menu;
mod splash;
mod ui_helper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = App::new();

    builder.insert_resource(ClearColor(Color::rgb(0., 0., 0.01)));

    builder.add_plugins({
        let builder = DefaultPlugins.build();
        let builder = builder
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "Spaceviata".to_string(),
                    ..default()
                },
                ..default()
            })
            .set(ImagePlugin::default_nearest());
        #[cfg(feature = "bundled")]
        {
            builder.add_before::<bevy::asset::AssetPlugin, _>(
                bevy_embedded_assets::EmbeddedAssetPlugin,
            );
        }
        builder
    });

    builder
        .add_plugin(bevy_easings::EasingsPlugin)
        .add_plugin(bevy_ninepatch::NinePatchPlugin::<()>::default());

    if cfg!(debug_assertions) {
        builder
            .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::filtered(vec![
                bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS,
            ]));
    }

    builder
        // game management
        .add_startup_system(general_setup)
        // ui
        .add_plugin(crate::ui_helper::button::Plugin)
        // screens
        .add_state(GameState::Splash)
        .add_state_to_stage(CoreStage::PostUpdate, GameState::Splash)
        .add_system_set(SystemSet::on_enter(GameState::Exit).with_system(exit))
        .add_plugin(crate::assets::AssetPlugin)
        .add_plugin(crate::splash::Plugin)
        .add_plugin(crate::menu::Plugin)
        // .add_plugin(crate::game::Plugin)
        .add_plugin(crate::lost::Plugin);
    #[cfg(feature = "debug-graph")]
    bevy_mod_debugdump::print_schedule(&mut builder);

    #[cfg(not(feature = "debug-graph"))]
    builder.run();

    Ok(())
}

#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub(crate) enum GameState {
    Splash,
    Menu,
    Playing,
    // Paused,
    Lost,
    Exit,
}

fn general_setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 10.0)),
        ..default()
    });
}

fn exit(mut app_exit_events: EventWriter<AppExit>) {
    app_exit_events.send(AppExit);
}
