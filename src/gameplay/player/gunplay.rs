use std::time::Duration;

use crate::{
    audio::sound_effect,
    gameplay::{
        crosshair::CrosshairState,
        health::{Death, Health},
        npc::Npc,
    },
    third_party::avian3d::CollisionLayer,
};

use super::{assets::PlayerAssets, camera::PlayerCamera, default_input::Shoot};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::events::Started;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Shooting;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Reloading;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shooting);
    app.add_observer(shooting_sounds);
    app.add_observer(handle_hits);
    app.add_observer(on_death);
    app.add_observer(shooting_sounds_reload);
    // Only until the animations work again.
    app.add_systems(Update, remove_shooting);
    app.add_systems(Update, trigger_reload_sound);
}

fn shooting(
    trigger: Trigger<Started<Shoot>>,
    mut commands: Commands,
    shooting: Query<(), With<Shooting>>,
    crosshair_state: Single<&CrosshairState>,
) {
    let entity = trigger.target();

    if shooting.contains(entity) || !crosshair_state.wants_invisible.is_empty() {
        return;
    }

    commands.entity(entity).insert(Shooting);
}

fn remove_shooting(
    shooting: Single<Entity, (With<Shooting>, With<Reloading>)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    mut commands: Commands,
) {
    let reload_time = 500;
    let timer = timer.get_or_insert_with(|| {
        Timer::new(Duration::from_millis(reload_time), TimerMode::Repeating)
    });
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }

    commands.entity(*shooting).remove::<Shooting>();
    commands.entity(*shooting).remove::<Reloading>();
}

fn trigger_reload_sound(
    shooting: Single<Entity, With<Shooting>>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    mut commands: Commands,
) {
    let shooting_sound_len = 200;
    let timer = timer.get_or_insert_with(|| {
        Timer::new(
            Duration::from_millis(shooting_sound_len),
            TimerMode::Repeating,
        )
    });
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }

    commands.entity(*shooting).insert(Reloading);
}

fn shooting_sounds(
    _trigger: Trigger<OnAdd, Shooting>,
    mut commands: Commands,
    mut player_assets: ResMut<PlayerAssets>,
) {
    let rng = &mut rand::thread_rng();
    let shooting_sound = player_assets.shooting_sounds.pick(rng).clone();

    commands.spawn(sound_effect(shooting_sound));
}

fn shooting_sounds_reload(
    _trigger: Trigger<OnAdd, Reloading>,
    mut commands: Commands,
    player_assets: ResMut<PlayerAssets>,
) {
    commands.spawn(sound_effect(player_assets.reload_sound.clone()));
}

fn handle_hits(
    _trigger: Trigger<OnAdd, Shooting>,
    spatial_query: SpatialQuery,
    player_camera_parent: Single<&Transform, With<PlayerCamera>>,
    mut npcs: Query<&mut Health, With<Npc>>,
) {
    // Ray origin and direction
    let origin = player_camera_parent.translation;
    let direction = player_camera_parent.forward();

    // Configuration for the ray cast
    let max_distance = 100.0;
    let solid = true;
    let filter =
        SpatialQueryFilter::default().with_mask([CollisionLayer::Npc, CollisionLayer::Prop]);

    // Cast ray and print first hit
    let Some(first_hit) = spatial_query.cast_ray(origin, direction, max_distance, solid, &filter)
    else {
        return;
    };
    let Ok(mut health) = npcs.get_mut(first_hit.entity) else {
        return;
    };

    let gun_damage = 10.0;
    health.damage(gun_damage);
}

fn on_death(trigger: Trigger<Death>, name: Query<NameOrEntity>) {
    let entity = trigger.target();
    let name = name.get(entity).unwrap();
    info!("Just died: {name}");
}
