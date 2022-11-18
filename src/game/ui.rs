use bevy::prelude::*;

use crate::{assets::UiAssets, ui_helper::button::ButtonId, GameState};

use super::world::{CameraController, CameraControllerTarget};

pub(crate) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system_set(SystemSet::on_enter(GameState::Game).with_system(setup))
            .add_system_set(SystemSet::on_update(GameState::Game).with_system(button_system));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum UiButtons {
    ZoomIn,
    ZoomOut,
}

impl From<UiButtons> for String {
    fn from(button: UiButtons) -> Self {
        match button {
            UiButtons::ZoomIn => {
                material_icons::icon_to_char(material_icons::Icon::ZoomIn).to_string()
            }
            UiButtons::ZoomOut => {
                material_icons::icon_to_char(material_icons::Icon::ZoomOut).to_string()
            }
        }
    }
}

#[derive(Component)]
struct LiveMarker;

#[derive(Component)]
struct CreditsMarker;

fn setup(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
) {
    info!("loading UI");

    let button_handle = ui_handles.button_handle.clone_weak();
    let button = buttons.get(&button_handle).unwrap();
    let material = ui_handles.font_material.clone_weak();

    let zoom_in_button = button.add(
        &mut commands,
        Val::Px(40.),
        Val::Px(40.),
        UiRect::all(Val::Auto),
        material.clone(),
        UiButtons::ZoomIn,
        30.,
    );
    let zoom_out_button = button.add(
        &mut commands,
        Val::Px(40.),
        Val::Px(40.),
        UiRect::all(Val::Auto),
        material,
        UiButtons::ZoomOut,
        30.,
    );

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            commands
                .spawn(NodeBundle {
                    style: Style {
                        position: UiRect {
                            right: Val::Px(20.0),
                            top: Val::Px(20.0),
                            ..default()
                        },
                        size: Size {
                            width: Val::Px(100.0),
                            height: Val::Px(40.0),
                        },
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|builder| {
                    builder
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            ..default()
                        })
                        .push_children(&[zoom_in_button, zoom_out_button]);
                });
        });
}

fn button_system(
    interaction_query: Query<(&Interaction, &ButtonId<UiButtons>, Changed<Interaction>)>,
    controller: Res<CameraController>,
    mut target: ResMut<CameraControllerTarget>,
) {
    for (interaction, button_id, changed) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match (button_id.0, changed) {
                (UiButtons::ZoomIn, _) => {
                    target.zoom_level = (controller.zoom_level + 1.0).min(10.0);
                    target.ignore = true;
                }
                (UiButtons::ZoomOut, _) => {
                    target.zoom_level = (controller.zoom_level - 1.0).max(1.0);
                    target.ignore = true;
                }
            }
        }
        if *interaction == Interaction::None && changed {
            target.ignore = false;
        }
    }
}
