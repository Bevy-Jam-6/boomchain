//! The main menu (seen on the title screen).

use bevy::{prelude::*, window::CursorGrabMode};

use crate::{
    font::FontAssets, gameplay::waves::GameMode, menus::Menu, screens::Screen, theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, fonts: Res<FontAssets>) {
    commands.spawn((
        widget::ui_root("Main Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Main),
        #[cfg(not(target_family = "wasm"))]
        children![
            (
                Text::new("Chainboom"),
                TextFont::from_font_size(52.0).with_font(fonts.default.clone())
            ),
            widget::button("Play", fonts.default.clone(), enter_loading_screen),
            widget::button(
                "Endless Mode",
                fonts.default.clone(),
                enter_loading_screen_endless
            ),
            widget::button("Settings", fonts.default.clone(), open_settings_menu),
            widget::button("Credits", fonts.default.clone(), open_credits_menu),
            widget::button("Exit", fonts.default.clone(), exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            (
                Text::new("Chainboom"),
                TextFont::from_font_size(52.0).with_font(fonts.default.clone())
            ),
            widget::button("Play", fonts.default.clone(), enter_loading_screen),
            widget::button(
                "Endless Mode",
                fonts.default.clone(),
                enter_loading_screen_endless
            ),
            widget::button("Settings", fonts.default.clone(), open_settings_menu),
            widget::button("Credits", fonts.default.clone(), open_credits_menu),
        ],
    ));
}

fn enter_loading_screen(
    _trigger: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_game_mode: ResMut<NextState<GameMode>>,
    mut window: Single<&mut Window>,
) {
    next_screen.set(Screen::Loading);
    next_game_mode.set(GameMode::Normal);
}

fn enter_loading_screen_endless(
    _trigger: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_game_mode: ResMut<NextState<GameMode>>,
    mut window: Single<&mut Window>,
) {
    next_screen.set(Screen::Loading);
    next_game_mode.set(GameMode::Endless);
}

fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
