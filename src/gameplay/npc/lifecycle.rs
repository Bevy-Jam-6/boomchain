use std::time::Duration;

use avian3d::prelude::*;
use bevy::{pbr::NotShadowCaster, prelude::*, scene::SceneInstanceReady};
use bevy_shuffle_bag::ShuffleBag;
use rand::Rng;

use crate::{
    despawn_after::{Despawn, DespawnAfter},
    gameplay::{
        explosion::{ExplodeOnDeath, OnExplode},
        health::{OnDamage, OnDeath},
        npc::{ai_state::AiState, assets::NpcAssets, stats::NpcStats},
    },
    screens::Screen,
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_enemy_death);
    app.add_observer(stagger_on_hit);
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Gib;

fn on_enemy_death(
    trigger: Trigger<OnDeath>,
    enemies: Query<(&Transform, &NpcStats, Has<ExplodeOnDeath>)>,
    npc_assets: Res<NpcAssets>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok((transform, stats, explode_on_death)) = enemies.get(entity) else {
        return;
    };
    let mut rng = rand::thread_rng();
    let mut gibs = ShuffleBag::try_new(
        [
            &npc_assets.gib_head,
            &npc_assets.gib_torso,
            &npc_assets.gib_arm_1,
            &npc_assets.gib_arm_2,
            &npc_assets.gib_arm_1,
            &npc_assets.gib_arm_2,
            &npc_assets.gib_leg,
            &npc_assets.gib_leg,
            &npc_assets.gib_foot,
            &npc_assets.gib_foot,
            &npc_assets.gib_pelvis,
        ],
        &mut rng,
    )
    .unwrap();
    let num_gibs = 5;
    for _ in 0..num_gibs {
        let gib = *gibs.pick(&mut rng);
        let offset_radius = 0.5;
        let offset = Sphere::new(offset_radius).sample_interior(&mut rng);
        let position = transform.translation + offset;
        commands
            .spawn((
                Gib,
                SceneRoot(gib.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(stats.size)),
                RigidBody::Dynamic,
                ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
                    .with_default_layers(CollisionLayers::new(
                        CollisionLayer::Gib,
                        [CollisionLayer::Default],
                    )),
                DespawnAfter::new(Duration::from_secs(10)),
                StateScoped(Screen::Gameplay),
            ))
            .observe(remove_shadow_caster);
    }

    commands.entity(entity).insert(Despawn);

    if explode_on_death {
        commands.entity(entity).trigger(OnExplode);
    }
}

fn remove_shadow_caster(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    mesh: Query<(), With<Mesh3d>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    for child in children.iter_descendants(entity) {
        if mesh.contains(child) {
            commands.entity(child).insert(NotShadowCaster);
        }
    }
}

fn stagger_on_hit(trigger: Trigger<OnDamage>, mut enemies: Query<(&mut AiState, &NpcStats)>) {
    let entity = trigger.target();
    let Ok((mut ai_state, stats)) = enemies.get_mut(entity) else {
        return;
    };
    if !matches!(*ai_state, AiState::Chase) {
        return;
    }

    if rand::thread_rng().gen_bool(stats.stagger_chance as f64) {
        let duration = rand::thread_rng().gen_range(stats.stagger_duration.clone());
        *ai_state = AiState::Stagger(Timer::from_seconds(duration, TimerMode::Once));
    }
}
