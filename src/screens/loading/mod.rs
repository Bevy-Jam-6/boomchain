//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;
use bevy_mesh_decal::Decal;

mod preload_assets;
mod shader_compilation;
mod spawn_level;

use crate::{
    gameplay::npc::{Npc, lifecycle::Gib},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<LoadingScreen>();
    app.add_plugins((
        shader_compilation::plugin,
        preload_assets::plugin,
        spawn_level::plugin,
    ));
    app.add_systems(OnEnter(Screen::Loading), clear_scene);
}

/// The game's main screen states.
#[derive(SubStates, Debug, Hash, PartialEq, Eq, Clone, Default)]
#[source(Screen = Screen::Loading)]
#[states(scoped_entities)]
pub(crate) enum LoadingScreen {
    #[default]
    Assets,
    Shaders,
    Level,
}

fn clear_scene(
    mut commands: Commands,
    entities: Query<Entity, Or<(With<Npc>, With<Gib>, With<Decal>)>>,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
