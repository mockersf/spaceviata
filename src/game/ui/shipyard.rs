use bevy::{prelude::*, ui::FocusPolicy};

use crate::{assets::UiAssets, game::world::CameraControllerTarget, ui_helper::button::ButtonId};

use super::{OneFrameDelay, ScreenTag, SelectedStar, DAMPENER};

#[derive(Clone, Copy)]
pub(crate) enum ShipyardButtons {
    Exit,
}

impl From<ShipyardButtons> for String {
    fn from(button: ShipyardButtons) -> Self {
        match button {
            ShipyardButtons::Exit => {
                material_icons::icon_to_char(material_icons::Icon::Logout).to_string()
            }
        }
    }
}

pub(crate) enum ShipyardEvent {
    OpenForStar(usize),
    Close,
}

#[derive(Component)]
pub(crate) struct ShipyardPanelMarker;

pub(crate) fn display_shipyard(
    mut commands: Commands,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    mut shipyard_events: EventReader<ShipyardEvent>,
    panel: Query<Entity, With<ShipyardPanelMarker>>,
    mut target: ResMut<CameraControllerTarget>,
    mut selected_star: ResMut<SelectedStar>,
) {
    match shipyard_events.iter().last() {
        Some(ShipyardEvent::OpenForStar(_index)) => {
            target.ignore_movement = true;
            selected_star.index = None;
            dbg!(target.ignore_movement);
            let button_handle = ui_handles.button_handle.clone_weak();
            let button = buttons.get(&button_handle).unwrap();

            let next_message_button = button.add_hidden(
                &mut commands,
                Val::Px(30.),
                Val::Px(30.),
                UiRect::all(Val::Auto),
                ui_handles.font_material.clone_weak(),
                ShipyardButtons::Exit,
                20.,
                crate::ui_helper::ColorScheme::TEXT,
                true,
            );
            let base = commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            size: Size {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                            },
                            display: Display::None,
                            ..Default::default()
                        },

                        ..Default::default()
                    },
                    OneFrameDelay,
                ))
                .with_children(|parent| {
                    parent
                        .spawn((
                            NodeBundle {
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: UiRect {
                                        right: Val::Px(0.0),
                                        bottom: Val::Px(0.0),
                                        ..default()
                                    },
                                    display: Display::None,
                                    ..default()
                                },
                                ..default()
                            },
                            OneFrameDelay,
                        ))
                        .push_children(&[next_message_button]);
                })
                .id();

            let panel_style = Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                size: Size::new(Val::Px(600.0), Val::Px(400.0)),
                align_content: AlignContent::Stretch,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            };

            let message_panel = commands
                .spawn(bevy_ninepatch::NinePatchBundle {
                    style: panel_style,
                    nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                        ui_handles.panel_handle.1.clone_weak(),
                        ui_handles.panel_handle.0.clone_weak(),
                        base,
                    ),
                    ..default()
                })
                .id();

            commands
                .spawn((
                    NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Auto),
                            size: Size::new(Val::Px(600.0), Val::Px(600.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        z_index: ZIndex::Global(10),
                        ..default()
                    },
                    ShipyardPanelMarker,
                    ScreenTag,
                ))
                .with_children(|parent| {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: BackgroundColor(DAMPENER),
                            ..default()
                        })
                        .push_children(&[message_panel]);
                });
        }
        Some(ShipyardEvent::Close) => {
            if let Ok(entity) = panel.get_single() {
                target.ignore_movement = false;
                commands.entity(entity).despawn_recursive();
            }
        }
        None => (),
    }
}

pub(crate) fn button_system(
    interaction_query: Query<(&Interaction, &ButtonId<ShipyardButtons>), Changed<Interaction>>,
    mut shipyard_events: EventWriter<ShipyardEvent>,
) {
    for (interaction, button_id) in interaction_query.iter() {
        if *interaction == Interaction::Clicked {
            match button_id.0 {
                ShipyardButtons::Exit => shipyard_events.send(ShipyardEvent::Close),
            }
        }
    }
}
