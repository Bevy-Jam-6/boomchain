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
use rand::prelude::*;

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
    let reload_time = 410;
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
    // The name is not precise, we simply start the reload time after this time of the shooting sound (they overlap a little)
    let shooting_sound_len = 175;
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
    let mut rng = &mut rand::thread_rng();

    // Ray origin and base direction
    let origin = player_camera_parent.translation;
    let base_direction = player_camera_parent.forward();

    // Spread configuration - adjust this value to control spread amount (in radians)
    // According to Claude Sonnet 4:
    // 0.05 = small spread (good for rifles)
    // 0.1 = medium spread (good for pistols)
    // 0.2 = large spread (good for shotguns)
    let spread_radius = 0.2;

    for i in 1..=8 {
        // Sample random point within a circle for spread
        let (offset_x, offset_y) = sample_circle(&mut rng, spread_radius);

        // Create perpendicular vectors to the forward direction for spreading
        let right = player_camera_parent.right();
        let up = player_camera_parent.up();

        // Apply spread to the direction
        let spread_vec = base_direction.as_vec3() + right * offset_x + up * offset_y;
        let spread_direction = Dir3::new(spread_vec).unwrap_or(Dir3::NEG_Z); // TODO: Is NEG_Z good enough?

        // Configuration for the ray cast
        let max_distance = 100.0;
        let solid = true;
        let filter =
            SpatialQueryFilter::default().with_mask([CollisionLayer::Npc, CollisionLayer::Prop]);

        // Cast ray with spread and handle first hit
        let Some(first_hit) =
            spatial_query.cast_ray(origin, spread_direction, max_distance, solid, &filter)
        else {
            return;
        };
        let Ok(mut health) = npcs.get_mut(first_hit.entity) else {
            return;
        };

        info!("Hit {i}/8 did hit {:?}", first_hit.entity);

        let gun_damage = 10.0;
        health.damage(gun_damage);
    }
}

/// Sample a random point within a circle using uniform distribution
/// This was AI generated using Claude Sonnet 4. I have no clue if it's correct tbh.
fn sample_circle(rng: &mut ThreadRng, radius: f32) -> (f32, f32) {
    // Generate random angle
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);

    // Generate random radius with square root for uniform distribution
    let r = radius * rng.r#gen::<f32>().sqrt();

    // Convert to Cartesian coordinates
    (r * angle.cos(), r * angle.sin())
}

fn on_death(trigger: Trigger<Death>, name: Query<NameOrEntity>) {
    let entity = trigger.target();
    let name = name.get(entity).unwrap();
    info!("Just died: {name}");
}
