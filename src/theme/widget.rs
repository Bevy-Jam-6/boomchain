//! Helper functions for creating common widgets.

use std::borrow::Cow;

use bevy::{
    ecs::{spawn::SpawnWith, system::IntoObserverSystem},
    prelude::*,
    ui::Val::*,
};

use crate::theme::{interaction::InteractionPalette, palette::*};

/// A root UI node that fills the window and centers its content.
pub(crate) fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: Percent(100.0),
            height: Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: Px(20.0),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
    )
}

/// A simple header label. Bigger than [`label`].
pub(crate) fn header(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::from_font_size(40.0).with_font(font),
        TextColor(HEADER_TEXT),
    )
}

/// A simple text label.
pub(crate) fn label(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    label_base(text, 24.0, font)
}

pub(crate) fn label_small(text: impl Into<String>, font: Handle<Font>) -> impl Bundle {
    label_base(text, 12.0, font)
}

/// A simple text label.
fn label_base(text: impl Into<String>, font_size: f32, font: Handle<Font>) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::from_font_size(font_size).with_font(font),
        TextColor(LABEL_TEXT),
    )
}

/// A large rounded button with text and an action defined as an [`Observer`].
pub(crate) fn button<E, B, M, I>(
    text: impl Into<String>,
    font: Handle<Font>,
    action: I,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        font,
        action,
        (
            Node {
                width: Px(380.0),
                height: Px(80.0),
                border: UiRect::all(Px(3.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::all(Px(5.0)),
        ),
    )
}

/// A small square button with text and an action defined as an [`Observer`].
pub(crate) fn button_small<E, B, M, I>(
    text: impl Into<String>,
    font: Handle<Font>,
    action: I,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    button_base(
        text,
        font,
        action,
        (
            Node {
                width: Px(30.0),
                height: Px(30.0),
                border: UiRect::all(Px(2.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BorderRadius::all(Px(5.0)),
        ),
    )
}

/// A simple button with text and an action defined as an [`Observer`]. The button's layout is provided by `button_bundle`.
fn button_base<E, B, M, I>(
    text: impl Into<String>,
    font: Handle<Font>,
    action: I,
    button_bundle: impl Bundle,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    Button,
                    BackgroundColor(BUTTON_BACKGROUND),
                    InteractionPalette {
                        none: BUTTON_BORDER,
                        hovered: BUTTON_HOVERED_BORDER,
                        pressed: BUTTON_PRESSED_BORDER,
                    },
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextLayout::new_with_justify(JustifyText::Center),
                        TextFont::from_font_size(32.0).with_font(font),
                        TextColor(BUTTON_TEXT),
                        // Don't bubble picking events from the text up to the button.
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_bundle)
                .observe(action);
        })),
    )
}

pub(crate) fn plus_minus_bar<E, B, M, I1, I2>(
    label_marker: impl Component,
    lower: I1,
    raise: I2,
    font: Handle<Font>,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I1: IntoObserverSystem<E, B, M>,
    I2: IntoObserverSystem<E, B, M>,
{
    (
        Node {
            justify_self: JustifySelf::Start,
            column_gap: Px(5.0),
            ..default()
        },
        children![
            button_small("-", font.clone(), lower),
            button_small("+", font.clone(), raise),
            (
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(label("", Handle::default()), label_marker)],
            ),
        ],
    )
}
