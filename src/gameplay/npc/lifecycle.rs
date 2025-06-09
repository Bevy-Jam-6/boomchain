use avian3d::prelude::*;
use bevy::{
    audio::{SpatialScale, Volume},
    pbr::NotShadowCaster,
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_shuffle_bag::ShuffleBag;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use rand::Rng;

use crate::{
    audio::SoundEffect,
    despawn_after::{Despawn, DespawnAfter},
    gameplay::{
        explosion::{ExplodeOnDeath, OnExplode},
        gore_settings::{Gore, GoreSettings},
        health::{OnDamage, OnDeath},
        npc::{ai_state::AiState, assets::NpcAssets, stats::NpcStats},
    },
    screens::Screen,
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_enemy_death);
    app.add_observer(stagger_on_hit);
    app.add_systems(Update, grunt_passively.run_if(in_state(Screen::Gameplay)));
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct Gib;

#[cfg_attr(feature = "hot_patch", hot)]
fn on_enemy_death(
    trigger: Trigger<OnDeath>,
    enemies: Query<(&Transform, &NpcStats, Has<ExplodeOnDeath>)>,
    npc_assets: Res<NpcAssets>,
    gore_settings: Res<GoreSettings>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok((transform, stats, explode_on_death)) = enemies.get(entity) else {
        return;
    };
    if gore_settings.gibs != Gore::None {
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

        for _ in 0..gore_settings.gib_count {
            let gib = *gibs.pick(&mut rng);
            let offset_radius = 0.5;
            let offset = Sphere::new(offset_radius).sample_interior(&mut rng);
            let position = transform.translation + offset;

            let mut entity_commands = commands.spawn((
                Gib,
                SceneRoot(gib.clone()),
                Transform::from_translation(position).with_scale(Vec3::splat(stats.size)),
                RigidBody::Dynamic,
                ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
                    .with_default_layers(CollisionLayers::new(
                        CollisionLayer::Gib,
                        [CollisionLayer::Default],
                    )),
                StateScoped(Screen::Gameplay),
            ));

            entity_commands.observe(remove_shadow_caster);

            if let Gore::Despawn(duration) = gore_settings.gibs {
                entity_commands.insert(DespawnAfter::new(duration));
            }
        }
    }

    commands.entity(entity).insert(Despawn);
    commands.entity(entity).queue_handled(
        |mut entity: EntityWorldMut| {
            entity.despawn_related::<Vocal>();
        },
        bevy::ecs::error::ignore,
    );

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

#[derive(Component, Deref, DerefMut, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Vocal)]
pub(crate) struct VocalOf(pub(crate) Entity);

#[derive(Component, Deref, DerefMut, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = VocalOf)]
pub(crate) struct Vocal(Entity);

#[cfg_attr(feature = "hot_patch", hot)]
fn grunt_passively(
    mut enemy: Query<(&AiState, &NpcStats, &Transform, Entity), Without<Vocal>>,
    mut commands: Commands,
    time: Res<Time>,
    mut npc_assets: ResMut<NpcAssets>,
) {
    for (ai_state, stats, transform, entity) in enemy.iter_mut() {
        if !matches!(*ai_state, AiState::Chase) {
            return;
        }

        let grunt_chance_per_second = 0.3;
        let grunt_chance = grunt_chance_per_second * time.delta_secs();
        if !rand::thread_rng().gen_bool(grunt_chance as f64) {
            continue;
        }

        let handle = npc_assets.idle_sound.pick(&mut rand::thread_rng()).clone();
        commands
            .spawn(enemy_sound_effect(handle, *transform, stats))
            .insert(VocalOf(entity));
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn stagger_on_hit(
    trigger: Trigger<OnDamage>,
    mut enemies: Query<(&mut AiState, &NpcStats, &Transform)>,
    mut commands: Commands,
    mut npc_assets: ResMut<NpcAssets>,
    state: Res<State<Screen>>,
) {
    if *state != Screen::Gameplay {
        return;
    }

    let entity = trigger.target();
    let Ok((mut ai_state, stats, transform)) = enemies.get_mut(entity) else {
        return;
    };
    if !matches!(*ai_state, AiState::Chase) {
        return;
    }

    if rand::thread_rng().gen_bool(stats.stagger_chance as f64) {
        let duration = rand::thread_rng().gen_range(stats.stagger_duration.clone());
        *ai_state = AiState::Stagger(Timer::from_seconds(duration, TimerMode::Once));
        let handle = npc_assets
            .stagger_sound
            .pick(&mut rand::thread_rng())
            .clone();
        commands.entity(entity).queue_handled(
            |mut entity: EntityWorldMut| {
                entity.despawn_related::<Vocal>();
            },
            bevy::ecs::error::ignore,
        );

        commands
            .spawn(enemy_sound_effect(handle, *transform, stats))
            .insert(VocalOf(entity));
    }
}

pub(crate) fn enemy_sound_effect(
    handle: Handle<AudioSource>,
    transform: Transform,
    stats: &NpcStats,
) -> impl Bundle {
    let speed_mod = rand::thread_rng().gen_range(0.9..1.1);
    (
        transform,
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN
            .with_spatial(true)
            .with_volume(Volume::Linear(0.9))
            .with_speed(1.0 / stats.size * speed_mod)
            .with_spatial_scale(SpatialScale::new(1.0 / 5.5)),
        SoundEffect,
    )
}
