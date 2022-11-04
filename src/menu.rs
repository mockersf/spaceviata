use std::time::Duration;

use bevy::{
    prelude::*,
    winit::{UpdateMode, WinitSettings},
};

use bevy_easings::Ease;

use crate::{
    assets::{CloneWeak, UiAssets},
    ui_helper::ColorScheme,
};

const CURRENT_STATE: crate::GameState = crate::GameState::Menu;

#[derive(Component)]
struct ScreenTag;

#[derive(Resource)]
struct Screen {
    first_load: bool,
    menu_selected: Option<i32>,
}
impl Default for Screen {
    fn default() -> Self {
        Screen {
            first_load: true,
            menu_selected: None,
        }
    }
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Screen::default())
            .add_system_set(SystemSet::on_enter(CURRENT_STATE).with_system(setup))
            .add_system_set(SystemSet::on_exit(CURRENT_STATE).with_system(tear_down))
            .add_system_set(
                SystemSet::on_update(CURRENT_STATE)
                    .with_system(keyboard_input_system)
                    .with_system(gamepad_input_system)
                    .with_system(button_system)
                    .with_system(display_menu_item_selector),
            );
    }
}

#[derive(Clone, Copy)]
enum MenuButton {
    NewGame,
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}

impl From<MenuButton> for String {
    fn from(button: MenuButton) -> String {
        match button {
            MenuButton::NewGame => "New Game".to_string(),
            #[cfg(not(target_arch = "wasm32"))]
            MenuButton::Quit => "quit".to_string(),
        }
    }
}

const MENU_BUTTONS: &[MenuButton] = &[
    MenuButton::NewGame,
    #[cfg(not(target_arch = "wasm32"))]
    MenuButton::Quit,
];

fn setup(
    mut commands: Commands,
    mut screen: ResMut<Screen>,
    ui_handles: Res<UiAssets>,
    buttons: Res<Assets<crate::ui_helper::button::Button>>,
    mut mouse_button_input: ResMut<Input<MouseButton>>,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut gamepad_input: ResMut<Input<GamepadButton>>,
    mut camera: Query<&mut Transform, With<Camera>>,
    windows: Res<Windows>,
) {
    info!("Loading screen");

    let mut transform = camera.single_mut();
    *transform = Transform::from_translation(Vec3::new(-1.0, 2.0, 10.0))
        .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);

    commands.insert_resource(WinitSettings {
        focused_mode: UpdateMode::Reactive {
            max_wait: Duration::from_secs_f32(1.0 / 30.0),
        },
        ..WinitSettings::desktop_app()
    });

    mouse_button_input.clear();
    keyboard_input.clear();
    gamepad_input.clear();

    let panel_handles = ui_handles.panel_handle.clone_weak();
    let button_handle = ui_handles.button_handle.clone_weak();
    let button = buttons.get(&button_handle).unwrap();
    let font = ui_handles.font_main.clone_weak();
    let font_details = ui_handles.font_sub.clone_weak();
    let menu_indicator = ui_handles.selection_handle.clone_weak();

    let height = windows.primary().height();

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Percent(15.),
                    right: Val::Undefined,
                    bottom: Val::Undefined,
                    top: Val::Percent(25.),
                },
                size: Size {
                    height: Val::Px(height / 5.0),
                    width: Val::Auto,
                },
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|title_parent| {
            title_parent.spawn(TextBundle {
                style: Style {
                    size: Size {
                        height: Val::Px(height / 5.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::from_section(
                    "Spaceviata".to_string(),
                    TextStyle {
                        font: font.clone(),
                        color: ColorScheme::TEXT,
                        font_size: height / 5.0,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
            title_parent.spawn(TextBundle {
                style: Style {
                    size: Size {
                        height: Val::Px(height / 20.0),
                        ..Default::default()
                    },
                    position: UiRect {
                        left: Val::Px(-height / 50.0),
                        ..default()
                    },
                    ..Default::default()
                },
                text: Text::from_section(
                    "The Space Opera".to_string(),
                    TextStyle {
                        font: font_details.clone(),
                        color: ColorScheme::TEXT,
                        font_size: height / 20.0,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
        })
        .insert(ScreenTag);

    let panel_style = Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            left: Val::Percent(50.),
            right: Val::Undefined,
            bottom: Val::Percent(15.),
            top: Val::Undefined,
        },
        margin: UiRect::all(Val::Px(0.)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        size: Size::new(
            Val::Px(height / 2.0),
            Val::Px(height / 8.0 * MENU_BUTTONS.len() as f32),
        ),
        align_content: AlignContent::Stretch,
        flex_direction: FlexDirection::Column,
        ..Default::default()
    };

    let buttons = MENU_BUTTONS
        .iter()
        .enumerate()
        .map(|(i, button_item)| {
            let entity = commands
                .spawn(NodeBundle {
                    style: Style {
                        margin: UiRect {
                            left: Val::Px(15.0),
                            right: Val::Auto,
                            top: Val::Auto,
                            bottom: Val::Auto,
                        },
                        flex_direction: FlexDirection::RowReverse,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .id();
            let button = button.add(
                &mut commands,
                height / 3.0,
                height / 15.0,
                UiRect::all(Val::Auto),
                font.clone(),
                *button_item,
                height / 30.0,
            );
            let indicator = commands
                .spawn((
                    ImageBundle {
                        style: Style {
                            size: Size {
                                height: Val::Px(17.),
                                width: Val::Px(17.),
                            },
                            margin: UiRect {
                                right: Val::Px(15.),
                                ..Default::default()
                            },
                            ..Default::default()
                        },

                        visibility: Visibility { is_visible: false },
                        image: UiImage(menu_indicator.clone()),
                        ..Default::default()
                    },
                    MenuItemSelector(i),
                ))
                .id();
            commands.entity(entity).push_children(&[button, indicator]);
            entity
        })
        .collect::<Vec<_>>();
    let inner_content = commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            ..Default::default()
        })
        .id();
    commands
        .entity(inner_content)
        .push_children(buttons.as_slice());

    let panel = commands
        .spawn((
            bevy_ninepatch::NinePatchBundle {
                style: panel_style.clone(),
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    panel_handles.1,
                    panel_handles.0,
                    inner_content,
                ),
                ..Default::default()
            },
            ScreenTag,
        ))
        .id();
    if screen.first_load {
        commands.entity(panel).insert((
            Style {
                position: UiRect {
                    left: Val::Percent(120.),
                    right: Val::Undefined,
                    bottom: Val::Percent(15.),
                    top: Val::Undefined,
                },
                ..panel_style
            },
            Style {
                position: UiRect {
                    left: Val::Percent(120.),
                    right: Val::Undefined,
                    bottom: Val::Percent(15.),
                    top: Val::Undefined,
                },
                ..panel_style
            }
            .ease_to(
                panel_style,
                bevy_easings::EaseFunction::BounceOut,
                bevy_easings::EasingType::Once {
                    duration: std::time::Duration::from_millis(800),
                },
            ),
        ));
    } else {
        commands.entity(panel).insert(panel_style);
    }

    screen.first_load = false;
}

#[derive(Component)]
struct PlayerName;

fn tear_down(mut commands: Commands, query: Query<Entity, With<ScreenTag>>) {
    info!("tear down");

    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn gamepad_input_system(
    mut state: ResMut<State<crate::GameState>>,
    mut screen: ResMut<Screen>,
    gamepads: Res<Gamepads>,
    gamepad_input: Res<Input<GamepadButton>>,
    gamepad_axis: Res<Axis<GamepadAxis>>,
    mut delay: Local<Option<Timer>>,
    time: Res<Time>,
) {
    for gamepad in gamepads.iter() {
        if let Some(mut has_delay) = delay.take() {
            if !has_delay.tick(time.delta()).just_finished() {
                *delay = Some(has_delay);
            }
        } else if gamepad_input
            .just_released(GamepadButton::new(gamepad, GamepadButtonType::DPadDown))
            || gamepad_axis
                .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
                .unwrap_or_default()
                < -0.5
        {
            screen.menu_selected = Some(
                screen
                    .menu_selected
                    .map(|i| i32::min(MENU_BUTTONS.len() as i32 - 1, i + 1))
                    .unwrap_or(0),
            );
            *delay = Some(Timer::from_seconds(0.2, TimerMode::Once));
        } else if gamepad_input
            .just_released(GamepadButton::new(gamepad, GamepadButtonType::DPadUp))
            || gamepad_axis
                .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
                .unwrap_or_default()
                > 0.5
        {
            screen.menu_selected = Some(
                screen
                    .menu_selected
                    .map(|i| i32::max(0, i - 1))
                    .unwrap_or(0),
            );
            *delay = Some(Timer::from_seconds(0.2, TimerMode::Once));
        }

        if gamepad_input.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::South)) {
            match screen.menu_selected {
                Some(0) => {
                    let _ = state.set(crate::GameState::Playing);
                }
                Some(2) => {
                    let _ = state.set(crate::GameState::Exit);
                }
                _ => (),
            }
        }
    }
}

fn keyboard_input_system(
    mut state: ResMut<State<crate::GameState>>,
    mut screen: ResMut<Screen>,
    keyboard_input: Res<Input<KeyCode>>,
    mut wnds: ResMut<Windows>,
) {
    if keyboard_input.just_released(KeyCode::Escape) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = state.set(crate::GameState::Exit);
        }
    } else if keyboard_input.just_released(KeyCode::F) {
        let window = wnds.get_primary_mut().unwrap();
        match window.mode() {
            bevy::window::WindowMode::Windowed => {
                window.set_mode(bevy::window::WindowMode::BorderlessFullscreen)
            }
            _ => window.set_mode(bevy::window::WindowMode::Windowed),
        }
    } else if keyboard_input.just_released(KeyCode::Down) {
        screen.menu_selected = Some(
            screen
                .menu_selected
                .map(|i| i32::min(MENU_BUTTONS.len() as i32 - 1, i + 1))
                .unwrap_or(0),
        );
    } else if keyboard_input.just_released(KeyCode::Up) {
        screen.menu_selected = Some(
            screen
                .menu_selected
                .map(|i| i32::max(0, i - 1))
                .unwrap_or(0),
        );
    } else if keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::Return)
    {
        match screen.menu_selected {
            Some(0) => {
                let _ = state.set(crate::GameState::Playing);
            }
            Some(2) => {
                let _ = state.set(crate::GameState::Exit);
            }
            _ => (),
        }
    }
}

fn button_system(
    mut state: ResMut<State<crate::GameState>>,
    mut screen: ResMut<Screen>,
    mut interaction_query: Query<
        (
            &Button,
            &Interaction,
            &crate::ui_helper::button::ButtonId<MenuButton>,
        ),
        Changed<Interaction>,
    >,
) {
    for (_button, interaction, button_id) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => match button_id.0 {
                #[cfg(not(target_arch = "wasm32"))]
                MenuButton::Quit => {
                    let _ = state.set(crate::GameState::Exit);
                }
                MenuButton::NewGame => {
                    let _ = state.set(crate::GameState::Playing);
                }
            },
            Interaction::Hovered => match button_id.0 {
                MenuButton::NewGame => screen.menu_selected = Some(0),
                #[cfg(not(target_arch = "wasm32"))]
                MenuButton::Quit => screen.menu_selected = Some(1),
            },
            Interaction::None => (),
        }
    }
}

#[derive(Component)]
struct MenuItemSelector(usize);

fn display_menu_item_selector(
    screen: Res<Screen>,
    mut query: Query<(&MenuItemSelector, &mut Visibility)>,
) {
    if let Some(index_selected) = screen.menu_selected {
        for (selector, mut visible) in query.iter_mut() {
            if selector.0 == index_selected as usize {
                visible.is_visible = true;
            } else {
                visible.is_visible = false;
            }
        }
    }
}
