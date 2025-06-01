use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<AiState>();
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub(crate) enum AiState {
    #[default]
    Chase,
    Attack,
}
