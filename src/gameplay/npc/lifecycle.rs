use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_shuffle_bag::ShuffleBag;
use rand::Rng;

use crate::{
    despawn_after::{Despawn, DespawnAfter},
    gameplay::{
        health::{OnDamage, OnDeath},
        npc::{ai_state::AiState, assets::NpcAssets, stats::NpcStats},
    },
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_enemy_death);
    app.add_observer(stagger_on_hit);
}

fn on_enemy_death(
    trigger: Trigger<OnDeath>,
    enemies: Query<(&Transform, &NpcStats)>,
    npc_assets: Res<NpcAssets>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok((transform, stats)) = enemies.get(entity) else {
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
                SceneRoot(gib.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(stats.size)),
                RigidBody::Dynamic,
                ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
                    .with_default_layers(CollisionLayers::new(
                        CollisionLayer::Gib,
                        [CollisionLayer::Default],
                    )),
                DespawnAfter::new(Duration::from_secs(10)),
            ))
            .observe(remove_shadow_interactions);
    }
    commands.entity(entity).insert(Despawn);
}

fn remove_shadow_interactions(
    trigger: Trigger<SceneInstanceReady>,
    children: Query<&Children>,
    mesh: Query<(), With<Mesh3d>>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    for child in children.iter_descendants(entity) {
        if mesh.contains(child) {
            commands
                .entity(child)
                .insert((NotShadowCaster, NotShadowReceiver));
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
