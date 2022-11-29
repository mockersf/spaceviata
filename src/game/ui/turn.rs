use std::time::Duration;

use bevy::{prelude::*, ui::FocusPolicy};
use bevy_easings::{Ease, EaseFunction, EasingComponent, EasingType};

use crate::{
    assets::UiAssets,
    game::{
        turns::{Message, Turns},
        ui::{ScreenTag, UiButtons},
        world::CameraControllerTarget,
        Universe,
    },
    ui_helper::button::ButtonText,
};

use super::{OneFrameDelay, SelectedStar, DAMPENER};

#[derive(Component)]
pub(crate) struct EndTurnButton;

#[derive(Resource, Default)]
pub(crate) struct DisplayedMessage(pub(crate) usize);

#[derive(Component)]
pub(crate) struct MessagePanelMarker;

#[derive(Component)]
pub(crate) struct MessageContentMarker;

pub(crate) fn setup(
    commands: &mut Commands,
    ui_handles: &UiAssets,
    buttons: &Assets<crate::ui_helper::button::Button>,
) {
    let button_handle = ui_handles.button_handle.clone_weak();
    let button = buttons.get(&button_handle).unwrap();
    let material = ui_handles.font_material.clone_weak();

    let end_turn = button.add(
        commands,
        Val::Px(40.),
        Val::Px(40.),
        UiRect::all(Val::Auto),
        material,
        UiButtons::EndTurn,
        25.,
        crate::ui_helper::ColorScheme::TEXT,
    );

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position: UiRect {
                        right: Val::Px(20.0),
                        bottom: Val::Px(20.0),
                        ..default()
                    },
                    size: Size {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                    },
                    flex_direction: FlexDirection::Column,
                    position_type: PositionType::Absolute,
                    align_items: AlignItems::Baseline,
                    ..default()
                },
                ..default()
            },
            EndTurnButton,
            ScreenTag,
        ))
        .push_children(&[end_turn]);
}

pub(crate) fn display_messages(
    mut commands: Commands,
    turns: Res<Turns>,
    mut current_message: ResMut<DisplayedMessage>,
    ui_handles: Res<UiAssets>,
    panel: Query<Entity, With<MessagePanelMarker>>,
    mut content: Query<&mut Text, With<MessageContentMarker>>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    mut text: Query<(&mut Text, &ButtonText<UiButtons>), Without<MessageContentMarker>>,
    mut selected_star: ResMut<SelectedStar>,
    universe: Res<Universe>,
    mut controller_target: ResMut<CameraControllerTarget>,
    end_turn_button: Query<Entity, With<EndTurnButton>>,
) {
    if turns.is_changed() && !turns.messages.is_empty() {
        if let Ok(entity) = panel.get_single() {
            commands.entity(entity).despawn_recursive();
        };

        current_message.0 = 0;

        commands
            .entity(end_turn_button.single())
            .remove::<EasingComponent<BackgroundColor>>()
            .insert(BackgroundColor(Color::NONE));

        let button_handle = ui_handles.button_handle.clone_weak();
        let button = buttons.get(&button_handle).unwrap();

        let next_message_button = button.add_hidden(
            &mut commands,
            Val::Px(30.),
            Val::Px(30.),
            UiRect::all(Val::Auto),
            ui_handles.font_material.clone_weak(),
            if turns.messages.len() == 1 {
                UiButtons::LastMessage
            } else {
                UiButtons::NextMessage
            },
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
                parent.spawn((
                    TextBundle {
                        text: Text::from_sections(
                            turns.messages[current_message.0].as_sections(&ui_handles),
                        ),
                        style: Style {
                            size: Size {
                                width: Val::Undefined,
                                height: Val::Px(100.0),
                            },
                            ..default()
                        },
                        ..default()
                    },
                    MessageContentMarker,
                ));
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
                        Ease::ease_to(
                            BackgroundColor(Color::NONE),
                            BackgroundColor(Color::rgba(0.9, 0.9, 0.9, 0.5)),
                            EaseFunction::SineInOut,
                            EasingType::PingPong {
                                duration: Duration::from_millis(400),
                                pause: None,
                            },
                        ),
                        OneFrameDelay,
                    ))
                    .push_children(&[next_message_button]);
            })
            .id();

        let panel_style = Style {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            size: Size::new(Val::Px(300.0), Val::Px(200.0)),
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
                        position: UiRect {
                            right: Val::Px(0.0),
                            ..default()
                        },
                        position_type: PositionType::Absolute,
                        size: Size::new(Val::Px(400.0), Val::Percent(100.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                },
                MessagePanelMarker,
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
    if current_message.is_changed() {
        if current_message.0 == turns.messages.len() {
            let Ok(entity) = panel.get_single() else {
                return
            };
            commands.entity(entity).despawn_recursive();
            commands
                .entity(end_turn_button.single())
                .insert(Ease::ease_to(
                    BackgroundColor(Color::NONE),
                    BackgroundColor(Color::rgba(0.9, 0.9, 0.9, 0.5)),
                    EaseFunction::SineInOut,
                    EasingType::PingPong {
                        duration: Duration::from_millis(400),
                        pause: None,
                    },
                ));
        } else if current_message.0 > 0 {
            if current_message.0 == turns.messages.len() - 1 {
                for (mut text, button) in &mut text {
                    if button.0 == UiButtons::NextMessage {
                        text.sections[0].value =
                            material_icons::icon_to_char(material_icons::Icon::Done).to_string();
                    }
                }
            }
            content.single_mut().sections =
                turns.messages[current_message.0].as_sections(&ui_handles);
            match turns.messages[current_message.0] {
                Message::ColonyFounded { index, .. } => {
                    if selected_star.index != Some(index) {
                        selected_star.index = Some(index);
                    }
                    controller_target.zoom_level = 8.0;
                    controller_target.position = universe.galaxy[index].position;
                }
                Message::StarExplored { index, .. } => {
                    if selected_star.index != Some(index) {
                        selected_star.index = Some(index);
                    }
                    controller_target.zoom_level = 8.0;
                    controller_target.position = universe.galaxy[index].position;
                }
                Message::Story {
                    index: Some(index), ..
                } => {
                    if selected_star.index != Some(index) {
                        selected_star.index = Some(index);
                    }
                    controller_target.zoom_level = 8.0;
                    controller_target.position = universe.galaxy[index].position;
                }
                Message::Fight { index, .. } => {
                    if selected_star.index != Some(index) {
                        selected_star.index = Some(index);
                    }
                    controller_target.zoom_level = 8.0;
                    controller_target.position = universe.galaxy[index].position;
                }
                Message::Turn(_) | Message::Story { index: None, .. } => (),
            }
        }
    }
}
