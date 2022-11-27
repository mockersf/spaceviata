use bevy::render::color::Color;

pub(crate) mod button;

pub(crate) struct ColorScheme;

impl ColorScheme {
    pub(crate) const TEXT: Color = Color::rgb_linear(
        0xDE as f32 / 255.0,
        0xDE as f32 / 255.0,
        0xF1 as f32 / 255.0,
    );
    pub(crate) const TEXT_DARK: Color = Color::rgb_linear(
        0x5D as f32 / 255.0,
        0x53 as f32 / 255.0,
        0x6B as f32 / 255.0,
    );
    pub(crate) const TEXT_HIGHLIGHT: Color = Color::rgb_linear(
        0x20 as f32 / 255.0,
        0x20 as f32 / 255.0,
        0x48 as f32 / 255.0,
    );
}
