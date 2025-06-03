use std::time::Duration;

use crate::{
    RenderLayer,
    audio::sound_effect,
    gameplay::{
        crosshair::CrosshairState, health::Health, npc::Npc, player::camera_shake::OnTrauma,
    },
    third_party::avian3d::CollisionLayer,
};

use super::{Player, assets::PlayerAssets, camera::PlayerCamera, default_input::Shoot};
use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_enhanced_input::events::Started;
use bevy_hanabi::prelude::*;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Shooting;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Reloading;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
struct WeaponStats {
    pellets: u32,
    spread_radius: f32,
}

pub(super) fn plugin(app: &mut App) {
    // Only until there is a weapon pickup/selection system
    app.add_observer(setup_weapon_stats);

    app.add_observer(shooting);
    app.add_observer(shooting_sounds);
    app.add_observer(handle_hits);
    app.add_observer(shooting_sounds_reload);

    // Only until the animations work again.
    app.add_systems(Update, remove_shooting);
    app.add_systems(Update, trigger_reload_sound);
}

fn setup_weapon_stats(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    commands.entity(trigger.target()).insert(WeaponStats {
        pellets: 8,
        spread_radius: 0.2,
    });
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
    commands.trigger(OnTrauma(0.4));
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
    weapon_stats: Single<&WeaponStats, With<Player>>,
    player: Single<Entity, With<Player>>,
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
) {
    let mut rng = &mut rand::thread_rng();

    // Ray origin and base direction
    let origin = player_camera_parent.translation;
    let base_direction = player_camera_parent.forward();
    // Create perpendicular vectors to the forward direction for spreading
    let right = player_camera_parent.right();
    let up = player_camera_parent.up();

    for i in 1..=weapon_stats.pellets {
        // Sample random point within a circle for spread
        let point = Circle::new(weapon_stats.spread_radius).sample_interior(&mut rng);

        // Apply spread to the direction
        let spread_vec = base_direction.as_vec3() + right * point.x + up * point.y;
        let spread_direction = Dir3::new(spread_vec).unwrap_or(Dir3::NEG_Z);

        // Configuration for the ray cast
        let max_distance = 300.0;
        let solid = true;
        let filter = SpatialQueryFilter::default().with_excluded_entities([*player]);

        // Cast ray with spread and handle first hit
        let Some(first_hit) =
            spatial_query.cast_ray(origin, spread_direction, max_distance, solid, &filter)
        else {
            continue;
        };
        let bias = 0.1;
        commands.spawn((
            particle_bundle(&mut effects),
            Transform::from_translation(origin + spread_direction * (first_hit.distance - bias)),
        ));

        let Ok(mut health) = npcs.get_mut(first_hit.entity) else {
            continue;
        };

        info!("Hit {i}/8 did hit {:?}", first_hit.entity);

        let gun_damage = 10.0;
        health.damage(gun_damage);
    }
}

fn particle_bundle(effects: &mut Assets<EffectAsset>) -> impl Bundle {
    let effect_handle = setup_bullet_impact(effects);
    (
        ParticleEffect::new(effect_handle),
        RenderLayers::from(RenderLayer::PARTICLES),
    )
}
fn setup_bullet_impact(effects: &mut Assets<EffectAsset>) -> Handle<EffectAsset> {
    let mut color_gradient = Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.0, 0.0, 0.0, 1.0)); // solid black
    color_gradient.add_key(0.5, Vec4::new(0.0, 0.0, 0.0, 1.0)); // solid black
    color_gradient.add_key(1.0, Vec4::new(0.0, 0.0, 0.0, 0.0)); // fade to transparent

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.1));
    size_gradient.add_key(1.0, Vec3::splat(0.1)); // constant size (or fade if you want)

    let writer = ExprWriter::new();

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(1.5).expr(); // adjust for fade duration
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // No velocity, no acceleration
    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(0.).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(0.).expr(),
    };

    let orientation = OrientModifier {
        mode: OrientMode::ParallelCameraDepthPlane,
        ..default()
    };

    let mut module = writer.finish();

    let round = RoundModifier::ellipse(&mut module);

    effects.add(
        EffectAsset::new(1, SpawnerSettings::once(1.0.into()), module)
            .with_name("bullet_impact")
            .init(init_pos)
            .init(init_vel)
            .init(init_age)
            .init(init_lifetime)
            .render(ColorOverLifetimeModifier::new(color_gradient))
            .render(SizeOverLifetimeModifier {
                gradient: size_gradient,
                screen_space_size: false,
            })
            .render(orientation)
            .render(round),
    )
}
