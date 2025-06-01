use std::any::Any as _;

use bevy::prelude::*;

use crate::{
    gameplay::{
        crosshair::CrosshairState,
        health::Death,
        player::{Player, default_input::BlocksInput},
    },
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_player_death);
}

fn on_player_death(
    trigger: Trigger<Death>,
    player: Query<(), With<Player>>,
    mut crosshair: Single<&mut CrosshairState>,
    mut block_input: ResMut<BlocksInput>,
    mut commands: Commands,
) {
    if !player.contains(trigger.target()) {
        return;
    }
    commands.spawn((
        widget::ui_root("Game Over Menu"),
        GlobalZIndex(2),
        StateScoped(Screen::Gameplay),
        children![
            widget::header("Game Over"),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
    crosshair
        .wants_free_cursor
        .insert(on_player_death.type_id());
    block_input.insert(on_player_death.type_id());
}

fn quit_to_title(
    _trigger: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut crosshair: Single<&mut CrosshairState>,
    mut block_input: ResMut<BlocksInput>,
) {
    next_screen.set(Screen::Title);
    crosshair
        .wants_free_cursor
        .remove(&on_player_death.type_id());
    block_input.remove(&on_player_death.type_id());
}
