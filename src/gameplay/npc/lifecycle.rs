use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy, RigidBody};
use bevy::prelude::*;
use bevy_shuffle_bag::ShuffleBag;

use crate::gameplay::{
    health::OnDeath,
    npc::{Npc, assets::NpcAssets},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_enemy_death);
}

fn on_enemy_death(
    trigger: Trigger<OnDeath>,
    enemies: Query<&Transform, With<Npc>>,
    npc_assets: Res<NpcAssets>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Some(transform) = enemies.get(entity).ok() else {
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
        commands.spawn((
            SceneRoot(gib.clone()),
            Transform::from_translation(position),
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
        ));
    }
    commands.entity(entity).try_despawn();
}
