use std::any::Any as _;

use bevy::prelude::*;

use crate::{
    font::FontAssets,
    gameplay::{
        crosshair::CrosshairState,
        health::OnDeath,
        player::{Player, default_input::BlocksInput},
        time::GameplayTime,
    },
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_player_death);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct GameOverMenu;

fn on_player_death(
    trigger: Trigger<OnDeath>,
    player: Query<(), With<Player>>,
    mut crosshair: Single<&mut CrosshairState>,
    mut block_input: ResMut<BlocksInput>,
    fonts: Res<FontAssets>,
    gameplay_time: Res<GameplayTime>,
    mut commands: Commands,
    mut window: Single<&mut Window>,
) {
    if !player.contains(trigger.target()) {
        return;
    }
    window.cursor_options.visible = true;
    let elapsed_secs = gameplay_time.elapsed_secs();
    let minutes = (elapsed_secs / 60.0) as u32;
    let seconds = (elapsed_secs % 60.0) as u32;
    let milliseconds = (elapsed_secs * 1000.0) as u32 % 1000;
    commands.spawn((
        widget::ui_root("Game Over Menu"),
        StateScoped(Screen::Gameplay),
        GameOverMenu,
        children![
            widget::header("Game Over", fonts.default.clone()),
            widget::label(
                format!("Time: {minutes:02}:{seconds:02}.{milliseconds:03}"),
                fonts.default.clone()
            ),
            widget::button("Try Again", fonts.default.clone(), try_again),
            widget::button("Quit to Title", fonts.default.clone(), quit_to_title),
        ],
    ));
    crosshair
        .wants_free_cursor
        .insert(on_player_death.type_id());
    block_input.insert(on_player_death.type_id());
}

fn try_again(
    _trigger: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut crosshair: Single<&mut CrosshairState>,
    mut block_input: ResMut<BlocksInput>,
) {
    next_screen.set(Screen::Loading);
    crosshair
        .wants_free_cursor
        .remove(&on_player_death.type_id());
    block_input.remove(&on_player_death.type_id());
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
