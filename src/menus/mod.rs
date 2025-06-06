//! The game's main screen states and transitions between them.

mod credits;
pub(crate) mod game_over;
pub(crate) mod game_won;
mod main;
mod pause;
mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        credits::plugin,
        main::plugin,
        settings::plugin,
        pause::plugin,
        game_over::plugin,
        game_won::plugin,
    ));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
#[states(scoped_entities)]
pub(crate) enum Menu {
    #[default]
    None,
    Main,
    Credits,
    Settings,
    Pause,
}
