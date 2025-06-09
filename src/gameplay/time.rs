use bevy::{prelude::*, time::Stopwatch};

use crate::{
    menus::{Menu, game_won::GameWonMenu},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GameplayTime>();
    app.init_resource::<GameplayTime>();

    app.add_systems(OnEnter(Screen::Gameplay), reset_gameplay_time);
    app.add_systems(
        Update,
        update_gameplay_time.run_if(
            in_state(Screen::Gameplay)
                .and(in_state(Menu::None))
                .and(|game_won_query: Query<&GameWonMenu>| game_won_query.is_empty()),
        ),
    );
}

#[derive(Resource, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub(crate) struct GameplayTime(pub Stopwatch);

fn reset_gameplay_time(mut gameplay_time: ResMut<GameplayTime>) {
    gameplay_time.reset();
}

fn update_gameplay_time(mut gameplay_time: ResMut<GameplayTime>, time: Res<Time>) {
    gameplay_time.tick(time.delta());
}
