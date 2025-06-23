use std::any::Any as _;

use bevy::prelude::*;

use crate::{
    audio::Music,
    font::FontAssets,
    gameplay::{
        crosshair::CrosshairState, player::default_input::BlocksInput, time::GameplayTime,
        waves::GameWon,
    },
    menus::assets::MenuAssets,
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct GameWonMusic;

fn on_game_won(
    _trigger: Trigger<GameWon>,
    mut crosshair: Single<&mut CrosshairState>,
    mut block_input: ResMut<BlocksInput>,
    fonts: Res<FontAssets>,
    menu_assets: Res<MenuAssets>,
    mut commands: Commands,
    game_won_marker: Query<(), With<GameWonMarker>>,
    gameplay_time: Res<GameplayTime>,
    mut window: Single<&mut Window>,
) {
    if !game_won_marker.is_empty() {
        return;
    }
    window.cursor_options.visible = true;
    let elapsed_secs = gameplay_time.elapsed_secs();
    let minutes = (elapsed_secs / 60.0) as u32;
    let seconds = (elapsed_secs % 60.0) as u32;
    let milliseconds = (elapsed_secs * 1000.0) as u32 % 1000;
    commands.spawn((GameWonMarker, StateScoped(Screen::Gameplay)));
    commands.spawn((
        widget::ui_root("Game Won Menu"),
        GlobalZIndex(2),
        StateScoped(Screen::Gameplay),
        children![
            widget::header(
                "Congratulations! You've won the game!",
                fonts.default.clone()
            ),
            widget::label(
                format!("Time: {minutes:02}:{seconds:02}.{milliseconds:03}"),
                fonts.default.clone()
            ),
            widget::button("Quit to Title", fonts.default.clone(), quit_to_title),
        ],
    ));
    commands.spawn((
        GameWonMusic,
        AudioPlayer::new(menu_assets.gamee_won_sound.clone()),
        PlaybackSettings::DESPAWN.with_speed(0.8),
        Music,
        StateScoped(Screen::Gameplay),
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
