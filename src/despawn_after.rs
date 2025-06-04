use std::time::Duration;

use bevy::prelude::*;

use crate::PostPhysicsAppSystems;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DespawnAfter>();
    app.register_type::<Despawn>();
    app.add_systems(
        Update,
        (despawn_after, despawn).in_set(PostPhysicsAppSystems::DespawnAfter),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct DespawnAfter(pub(crate) Timer);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct Despawn;

impl DespawnAfter {
    pub(crate) fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

fn despawn_after(
    mut commands: Commands,
    time: Res<Time>,
    mut to_despawn: Query<(&mut DespawnAfter, Entity), Without<Despawn>>,
) {
    for (mut despawn_after, entity) in to_despawn.iter_mut() {
        despawn_after.0.tick(time.delta());
        if despawn_after.0.finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

fn despawn(mut commands: Commands, to_despawn: Query<Entity, With<Despawn>>) {
    for entity in to_despawn.iter() {
        commands.entity(entity).try_despawn();
    }
}
