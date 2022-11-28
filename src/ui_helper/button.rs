use bevy::prelude::*;

use crate::game::ui::OneFrameDelay;

#[derive(bevy::reflect::TypeUuid)]
#[uuid = "5114f317-f6a6-4436-bd2a-cb380f5eb551"]
pub(crate) struct Button {
    nine_patch: Handle<bevy_ninepatch::NinePatchBuilder<()>>,
    texture: Handle<Image>,
}

#[derive(Component)]
pub(crate) struct ButtonId<T: Into<String>>(pub(crate) T);

#[derive(Component)]
pub(crate) struct ButtonText<T: Into<String>>(pub(crate) T);

impl Button {
    pub(crate) fn setup(
        nine_patches: &mut Assets<bevy_ninepatch::NinePatchBuilder>,
        texture_handle: Handle<Image>,
    ) -> Button {
        let nine_patch = bevy_ninepatch::NinePatchBuilder::by_margins(7, 7, 7, 7);
        Button {
            nine_patch: nine_patches.add(nine_patch),
            texture: texture_handle,
        }
    }

    pub(crate) fn add<T>(
        &self,
        commands: &mut Commands,
        width: Val,
        height: Val,
        margin: UiRect,
        font: Handle<Font>,
        button: T,
        font_size: f32,
        font_color: Color,
    ) -> Entity
    where
        T: Into<String> + Send + Sync + Copy + 'static,
    {
        self.add_hidden(
            commands, width, height, margin, font, button, font_size, font_color, false,
        )
    }

    pub(crate) fn add_hidden<T>(
        &self,
        commands: &mut Commands,
        width: Val,
        height: Val,
        margin: UiRect,
        font: Handle<Font>,
        button: T,
        font_size: f32,
        font_color: Color,
        hidden: bool,
    ) -> Entity
    where
        T: Into<String> + Send + Sync + Copy + 'static,
    {
        self.add_hidden_section(
            commands,
            width,
            height,
            margin,
            vec![TextSection {
                value: button.into(),
                style: TextStyle {
                    font,
                    font_size,
                    color: font_color,
                    ..Default::default()
                },
            }],
            button,
            font_size,
            hidden,
        )
    }

    pub(crate) fn add_hidden_section<T>(
        &self,
        commands: &mut Commands,
        width: Val,
        height: Val,
        margin: UiRect,
        sections: Vec<TextSection>,
        button: T,
        font_size: f32,
        hidden: bool,
    ) -> Entity
    where
        T: Into<String> + Send + Sync + Copy + 'static,
    {
        let button_entity = commands
            .spawn((
                ButtonBundle {
                    style: Style {
                        size: Size::new(width, height),
                        margin,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        display: if hidden { Display::None } else { Display::Flex },
                        ..Default::default()
                    },
                    background_color: Color::NONE.into(),
                    ..Default::default()
                },
                OneFrameDelay,
                ButtonId(button),
            ))
            .id();

        let button_content = commands
            .spawn((
                TextBundle {
                    style: Style {
                        size: Size {
                            height: Val::Px(font_size),
                            ..Default::default()
                        },
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        display: if hidden { Display::None } else { Display::Flex },

                        ..Default::default()
                    },
                    text: Text::from_sections(sections),
                    focus_policy: bevy::ui::FocusPolicy::Pass,
                    ..Default::default()
                },
                OneFrameDelay,
                ButtonText(button),
            ))
            .id();

        let patch_entity = commands
            .spawn(bevy_ninepatch::NinePatchBundle::<()> {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    size: Size::new(width, height),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                nine_patch_data: bevy_ninepatch::NinePatchData::with_single_content(
                    self.texture.clone(),
                    self.nine_patch.clone(),
                    button_content,
                ),
                ..Default::default()
            })
            .id();

        let interaction_overlay = commands
            .spawn(ImageBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    margin: UiRect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    size: Size::new(width, height),
                    ..Default::default()
                },
                focus_policy: bevy::ui::FocusPolicy::Pass,
                ..Default::default()
            })
            .id();

        commands
            .entity(button_entity)
            .push_children(&[patch_entity, interaction_overlay]);

        button_entity
    }
}

fn button_effect(
    interaction_query: Query<
        (&bevy::ui::widget::Button, &Interaction, &Children),
        Changed<Interaction>,
    >,
    mut image_query: Query<&mut BackgroundColor>,
) {
    for (_button, interaction, children) in interaction_query.iter() {
        if let Ok(mut material) =
            image_query.get_component_mut::<BackgroundColor>(children[children.len() - 1])
        {
            match *interaction {
                Interaction::Clicked => {
                    material.0 = Color::rgba(0., 0.2, 0.2, 0.6);
                }
                Interaction::Hovered => {
                    material.0 = Color::rgba(0., 0.2, 0.2, 0.3);
                }
                Interaction::None => {
                    material.0 = Color::NONE;
                }
            }
        }
    }
}

pub(crate) struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<Button>().add_system(button_effect);
    }
}
