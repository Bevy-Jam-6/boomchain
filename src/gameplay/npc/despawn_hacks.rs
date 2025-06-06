use bevy::prelude::*;

use crate::{
    PostPhysicsAppSystems,
    gameplay::{
        health::OnDamage,
        npc::{Npc, ai_state::AiState},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (despawn_lazy, despawn_lonely).in_set(PostPhysicsAppSystems::TickTimers),
    );
    app.add_observer(init_last_translation);
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Lazy {
    begin: Vec3,
    timer: Timer,
}

#[derive(Component, Debug, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub(crate) struct LastEnemyTranslation(Vec3);

fn init_last_translation(
    trigger: Trigger<OnAdd, Npc>,
    mut commands: Commands,
    transform: Query<&Transform>,
) {
    let transform = transform.get(trigger.target()).copied().unwrap_or_default();
    commands
        .entity(trigger.target())
        .insert(LastEnemyTranslation(transform.translation));
}

fn despawn_lazy(
    mut commands: Commands,
    mut enemies: Query<(
        Entity,
        &mut LastEnemyTranslation,
        &AiState,
        &Transform,
        Option<&mut Lazy>,
    )>,
    time: Res<Time>,
) {
    for (entity, mut last_translation_mut, ai_state, transform, lazy) in enemies.iter_mut() {
        let last_translation = **last_translation_mut;
        let translation = transform.translation;
        **last_translation_mut = translation;
        if let Some(mut lazy) = lazy {
            if lazy.begin.distance_squared(translation) > 1.0 || matches!(ai_state, AiState::Attack)
            {
                commands.entity(entity).remove::<Lazy>();
                continue;
            }
            lazy.timer.tick(time.delta());
            if lazy.timer.finished() {
                commands.entity(entity).trigger(OnDamage(1000.0));
            }
            continue;
        }
        if translation.distance_squared(last_translation) < 1.0
            && !matches!(ai_state, AiState::Attack)
        {
            commands.entity(entity).insert(Lazy {
                begin: translation,
                timer: Timer::from_seconds(5.0, TimerMode::Once),
            });
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Lonely(Timer);

impl Default for Lonely {
    fn default() -> Self {
        Self(Timer::from_seconds(6.0, TimerMode::Repeating))
    }
}

fn despawn_lonely(
    mut commands: Commands,
    enemies: Query<Entity, With<Npc>>,
    mut lonelies: Query<(Entity, &mut Lonely)>,
    time: Res<Time>,
) {
    let is_lonely = enemies.iter().count() == 1;
    if !is_lonely {
        for (entity, _lonely) in lonelies.iter() {
            commands.entity(entity).remove::<Lonely>();
        }
        return;
    }
    let Some(lonely_entity) = enemies.iter().next() else {
        error!("No lonely enemy found, but queue is equal to 1?!");
        return;
    };
    if let Ok((_entity, mut lonely)) = lonelies.get_mut(lonely_entity) {
        lonely.0.tick(time.delta());
        if lonely.0.finished() {
            commands.entity(lonely_entity).trigger(OnDamage(1000.0));
        }
    } else {
        commands.entity(lonely_entity).insert(Lonely::default());
    }
}
