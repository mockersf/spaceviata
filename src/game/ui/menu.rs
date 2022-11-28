use bevy::prelude::*;

use crate::assets::UiAssets;

use super::{ScreenTag, UiButtons};

#[derive(Component)]
pub(crate) struct MenuContainer;

pub(crate) fn setup(
    commands: &mut Commands,
    ui_handles: &UiAssets,
    buttons: &Assets<crate::ui_helper::button::Button>,
) {
    {
        let button_handle = ui_handles.button_handle.clone_weak();
        let button = buttons.get(&button_handle).unwrap();
        let material = ui_handles.font_material.clone_weak();

        let zoom_in_button = button.add(
            commands,
            Val::Px(40.),
            Val::Px(40.),
            UiRect::all(Val::Auto),
            material.clone(),
            UiButtons::ZoomIn,
            25.,
            crate::ui_helper::ColorScheme::TEXT,
        );
        let zoom_out_button = button.add(
            commands,
            Val::Px(40.),
            Val::Px(40.),
            UiRect::all(Val::Auto),
            material.clone(),
            UiButtons::ZoomOut,
            25.,
            crate::ui_helper::ColorScheme::TEXT,
        );
        let game_menu_button = button.add(
            commands,
            Val::Px(40.),
            Val::Px(40.),
            UiRect::all(Val::Auto),
            material,
            UiButtons::GameMenu,
            25.,
            crate::ui_helper::ColorScheme::TEXT,
        );
        let back_to_menu_button = button.add(
            commands,
            Val::Px(100.),
            Val::Px(40.),
            UiRect::all(Val::Auto),
            ui_handles.font_sub.clone_weak(),
            UiButtons::BackToMenu,
            25.,
            crate::ui_helper::ColorScheme::TEXT,
        );

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        position: UiRect {
                            right: Val::Px(20.0),
                            top: Val::Px(20.0),
                            ..default()
                        },
                        size: Size {
                            width: Val::Px(100.0),
                            height: Val::Px(150.0),
                        },
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceAround,
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    ..default()
                },
                ScreenTag,
            ))
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
            })
            .push_children(&[game_menu_button, back_to_menu_button])
            .with_children(|builder| {
                builder
                    .spawn((
                        NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::SpaceAround,
                                ..default()
                            },
                            visibility: Visibility::INVISIBLE,
                            ..default()
                        },
                        MenuContainer,
                    ))
                    .push_children(&[back_to_menu_button]);
            });
    }
}
