use std::any::Any as _;

use bevy::prelude::*;

use crate::{
    gameplay::{crosshair::CrosshairState, player::default_input::BlocksInput, waves::GameWon},
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_game_won);
}

#[derive(Component)]
struct GameWonMarker;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct GameWonMenu;

fn on_game_won(
    _trigger: Trigger<GameWon>,
    mut crosshair: Single<&mut CrosshairState>,
    mut block_input: ResMut<BlocksInput>,
    mut commands: Commands,
    game_won_marker: Query<(), With<GameWonMarker>>,
) {
    if !game_won_marker.is_empty() {
        return;
    }
    commands.spawn((GameWonMarker, StateScoped(Screen::Gameplay)));
    commands.spawn((
        widget::ui_root("Game Won Menu"),
        GlobalZIndex(2),
        StateScoped(Screen::Gameplay),
        GameWonMenu,
        children![
            widget::header("Congratulations! You've won the game!"),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
    crosshair.wants_free_cursor.insert(on_game_won.type_id());
    block_input.insert(on_game_won.type_id());
}

fn quit_to_title(
    _trigger: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut crosshair: Single<&mut CrosshairState>,
    mut block_input: ResMut<BlocksInput>,
) {
    next_screen.set(Screen::Title);
    crosshair.wants_free_cursor.remove(&on_game_won.type_id());
    block_input.remove(&on_game_won.type_id());
}
