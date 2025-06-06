use bevy::{
    color::palettes::tailwind::{GRAY_500, RED_400, RED_500},
    prelude::*,
};

pub(crate) const LABEL_TEXT: Color = Color::srgb(1.0, 0.9, 0.9);
pub(crate) const HEADER_TEXT: Color = Color::srgb(0.9, 0.5, 0.5);
pub(crate) const BUTTON_TEXT: Color = Color::srgb(1.0, 0.925, 0.925);

pub(crate) const BUTTON_BACKGROUND: Color = Color::BLACK;
pub(crate) const BUTTON_HOVERED_BACKGROUND: Color = Color::srgb(0.384, 0.600, 0.820);
pub(crate) const BUTTON_PRESSED_BACKGROUND: Color = Color::srgb(0.239, 0.286, 0.600);

pub(crate) const BUTTON_BORDER: Color = Color::Srgba(GRAY_500);
pub(crate) const BUTTON_HOVERED_BORDER: Color = Color::Srgba(RED_500);
pub(crate) const BUTTON_PRESSED_BORDER: Color = Color::Srgba(RED_400);

pub(crate) const SCREEN_BACKGROUND: Color = Color::srgb(0.035, 0.01, 0.01);
