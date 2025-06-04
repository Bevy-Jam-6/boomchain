use std::time::Duration;

use bevy::prelude::*;

use crate::PostPhysicsAppSystems;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DespawnAfter>();
    app.add_systems(
        Update,
        despawn_after.in_set(PostPhysicsAppSystems::DespawnAfter),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct DespawnAfter(pub(crate) Timer);

impl DespawnAfter {
    pub(crate) fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

fn despawn_after(
    mut commands: Commands,
    time: Res<Time>,
    mut to_despawn: Query<(&mut DespawnAfter, Entity)>,
) {
    for (mut despawn_after, entity) in to_despawn.iter_mut() {
        despawn_after.0.tick(time.delta());
        if despawn_after.0.finished() {
            commands.entity(entity).try_despawn();
        }
    }
}
