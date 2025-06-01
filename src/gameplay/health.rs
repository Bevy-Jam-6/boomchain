use bevy::{prelude::*, ui::Val::*};

use crate::{screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_health_hud);
}

fn spawn_health_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Health HUD"),
        GlobalZIndex(2),
        StateScoped(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            width: Percent(100.0),
            height: Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            bottom: Px(20.0),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
        children![widget::label("Health")],
    ));
}
