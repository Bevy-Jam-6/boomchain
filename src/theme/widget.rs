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
    button_font: Handle<Font>,
    label_font: Handle<Font>,
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
            button_small("-", button_font.clone(), lower),
            button_small("+", button_font.clone(), raise),
            (
                Node {
                    padding: UiRect::horizontal(Px(10.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                children![(label("", label_font.clone()), label_marker)],
            ),
        ],
    )
}

#[derive(Component, Clone, Default)]
pub(crate) struct SelectInput {
    pub options: Vec<String>,
    pub selection: usize,
}

#[derive(Component, Clone, Default)]
pub(crate) struct ActiveSelectionLabel;

#[derive(Event)]
pub(crate) struct OnChangeSelection {
    pub selection: usize,
}

impl OnChangeSelection {
    pub const fn new(selection: usize) -> Self {
        Self { selection }
    }
}

pub(crate) fn cycle_select<E, B, M, I>(
    options: Vec<String>,
    selection: usize,
    font: Handle<Font>,
    change_selection: I,
) -> impl Bundle
where
    E: Event,
    B: Bundle,
    I: IntoObserverSystem<E, B, M> + Sync,
{
    let first_option = options[selection].clone();
    (
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Select Input"),
                    SelectInput {
                        options,
                        selection: 0,
                    },
                    Node::default(),
                    children![
                        button_small("<", font.clone(), on_change_selection::<-1>),
                        (
                            Node {
                                width: Px(320.0),
                                padding: UiRect::horizontal(Px(10.0)),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![(label(first_option, font.clone()), ActiveSelectionLabel)],
                        ),
                        button_small(">", font.clone(), on_change_selection::<1>),
                    ],
                ))
                .observe(change_selection);
        })),
    )
}

fn on_change_selection<const INCREMENT: i32>(
    trigger: Trigger<Pointer<Click>>,
    mut select_query: Query<&mut SelectInput>,
    child_of_query: Query<&ChildOf>,
    child_query: Query<&Children>,
    mut text_query: Query<&mut Text, With<ActiveSelectionLabel>>,
    mut commands: Commands,
) {
    let Ok(Ok(select_entity)) = child_of_query
        .get(trigger.target())
        .map(|&ChildOf(parent)| child_of_query.get(parent).map(|c| c.0))
    else {
        return;
    };

    let Ok(mut input) = select_query.get_mut(select_entity) else {
        return;
    };

    // Increment or decrement the selected index, wrapping around the options.
    input.selection =
        (input.selection as i32 + INCREMENT).rem_euclid(input.options.len() as i32) as usize;

    // Find the text entity within the select input's children and update its text.
    if let Some(text_entity) = child_query
        .iter_descendants(select_entity)
        .find(|&child| text_query.contains(child))
    {
        let mut text = text_query.get_mut(text_entity).unwrap();
        text.0 = input.options[input.selection].clone();
    }

    // Trigger the `OnChangeSelection` event with the updated selected index.
    commands.trigger_targets(
        OnChangeSelection {
            selection: input.selection,
        },
        select_entity,
    );
}
