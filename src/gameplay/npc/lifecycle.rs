use bevy::prelude::*;

use crate::gameplay::{health::Death, npc::Npc};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_enemy_death);
}

fn on_enemy_death(trigger: Trigger<Death>, enemies: Query<(), With<Npc>>, mut commands: Commands) {
    let entity = trigger.target();
    if !enemies.contains(entity) {
        return;
    }
    commands.entity(entity).try_despawn();
}
