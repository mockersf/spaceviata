#![allow(clippy::needless_update, clippy::too_many_arguments)]

use assets::names::{Names, NamesLoader};
#[cfg(not(target_arch = "wasm32"))]
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::{app::AppExit, log::LogPlugin, prelude::*};
use bevy_prototype_lyon::prelude::ShapePlugin;

mod assets;
mod game;
mod lost;
mod menu;
mod splash;
mod ui_helper;

#[bevy_main]
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = App::new();

    builder.insert_resource(ClearColor(Color::rgb(0., 0., 0.01)));

    builder.add_plugins({
        let mut builder = DefaultPlugins.build();
        builder = builder
            .set(WindowPlugin {
                window: WindowDescriptor {
                    title: "Spaceviata".to_string(),
                    fit_canvas_to_parent: true,
                    #[cfg(target_os = "ios")]
                    resizable: false,
                    #[cfg(target_os = "ios")]
                    mode: WindowMode::BorderlessFullscreen,
                    ..default()
                },
                ..default()
            })
            .set(LogPlugin {
                filter: format!("winit=error,{}", LogPlugin::default().filter),
                ..default()
            });
        #[cfg(feature = "bundled")]
        {
            builder = builder.add_before::<bevy::asset::AssetPlugin, _>(
                bevy_embedded_assets::EmbeddedAssetPlugin,
            );
        }
        #[cfg(feature = "hot")]
        {
            builder = builder.set(AssetPlugin {
                // Tell the asset server to watch for asset changes on disk:
                watch_for_changes: true,
                ..default()
            });
        }
        builder.set(ImagePlugin::default_nearest())
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

    builder.add_asset::<Names>().add_asset_loader(NamesLoader);

    builder.add_plugin(ShapePlugin);

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
        .add_plugin(crate::game::setup::Plugin)
        .add_plugin(crate::game::world::Plugin)
        .add_plugin(crate::game::starfield::Plugin)
        .add_plugin(crate::game::ui::Plugin)
        .add_plugin(crate::game::turns::Plugin)
        .add_plugin(crate::game::fleet::Plugin)
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
    Setup,
    Game,
    Lost,
    Exit,
}

fn general_setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            #[cfg(not(target_arch = "wasm32"))]
            camera: Camera {
                hdr: true,
                ..default()
            },
            ..default()
        },
        #[cfg(not(target_arch = "wasm32"))]
        BloomSettings { ..default() },
    ));
}

fn exit(mut app_exit_events: EventWriter<AppExit>) {
    app_exit_events.send(AppExit);
}
