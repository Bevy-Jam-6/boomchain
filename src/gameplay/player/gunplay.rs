use std::time::Duration;

use crate::{
    RenderLayer,
    audio::{sound_effect, sped_up_sound_effect},
    despawn_after::DespawnAfter,
    gameplay::{
        crosshair::CrosshairState, health::OnDamage, npc::Npc, player::camera_shake::OnTrauma,
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
    damage: f32,
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
    app.init_resource::<BulletImpact>();
}

fn setup_weapon_stats(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    commands.entity(trigger.target()).insert(WeaponStats {
        damage: 5.0,
        pellets: 16,
        spread_radius: 0.15,
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
    collider_of: Query<&ColliderOf>,
    weapon_stats: Single<&WeaponStats, With<Player>>,
    player: Single<Entity, With<Player>>,
    bullet_impact: Res<BulletImpact>,
    mut commands: Commands,
    npcs: Query<(), With<Npc>>,
    mut player_assets: ResMut<PlayerAssets>,
) {
    let mut rng = &mut rand::thread_rng();

    // Ray origin and base direction
    let origin = player_camera_parent.translation;
    let base_direction = player_camera_parent.forward();
    // Create perpendicular vectors to the forward direction for spreading
    let right = player_camera_parent.right();
    let up = player_camera_parent.up();

    for _i in 1..=weapon_stats.pellets {
        // Sample random point within a circle for spread
        let point = Circle::new(weapon_stats.spread_radius).sample_interior(&mut rng);

        // Apply spread to the direction
        let spread_vec = base_direction.as_vec3() + right * point.x + up * point.y;
        let spread_direction = Dir3::new(spread_vec).unwrap_or(Dir3::NEG_Z);

        // Configuration for the ray cast
        let max_distance = 300.0;
        let solid = true;
        let filter = SpatialQueryFilter::default()
            .with_mask([
                CollisionLayer::Npc,
                CollisionLayer::Prop,
                CollisionLayer::Default,
            ])
            .with_excluded_entities([*player]);

        // Cast ray with spread and handle first hit
        let Some(first_hit) =
            spatial_query.cast_ray(origin, spread_direction, max_distance, solid, &filter)
        else {
            continue;
        };
        let bias = 0.1;
        commands.spawn((
            Name::new("bullet impact particles"),
            DespawnAfter::new(Duration::from_secs(2)),
            particle_bundle(&bullet_impact),
            Transform::from_translation(
                origin + spread_direction * (first_hit.distance - bias).max(0.0),
            ),
        ));

        if npcs.contains(first_hit.entity) {
            // play jump sound sped up, sound like flesh impact
            let rng = &mut rand::thread_rng();
            let sound = player_assets.jump_start_sounds.pick(rng).clone();
            commands.spawn(sped_up_sound_effect(sound.clone()));
        } else {
            // play throw sound sped up, sounds like wall impact
            let sound = player_assets.throw_sound.clone();
            commands.spawn(sped_up_sound_effect(sound.clone()));
        }

        let Ok(ColliderOf { body }) = collider_of.get(first_hit.entity) else {
            error!("Hit something without a rigid body");
            continue;
        };

        commands
            .entity(*body)
            .trigger(OnDamage(weapon_stats.damage));
    }
}

#[derive(Resource)]
struct BulletImpact(Handle<EffectAsset>);

impl FromWorld for BulletImpact {
    fn from_world(world: &mut World) -> Self {
        let mut effects = world.resource_mut::<Assets<EffectAsset>>();
        Self(effects.add(create_bullet_impact_asset()))
    }
}

fn particle_bundle(effect: &BulletImpact) -> impl Bundle {
    (
        ParticleEffect::new(effect.0.clone()),
        RenderLayers::from(RenderLayer::PARTICLES),
    )
}

fn create_bullet_impact_asset() -> EffectAsset {
    let writer = ExprWriter::new();

    // init
    let c = writer.lit(0.05).uniform(writer.lit(0.4));
    let rgb = writer.lit(Vec3::ONE) * c;
    let color = rgb.vec4_xyz_w(writer.lit(1.)).pack4x8unorm();
    let init_color = SetAttributeModifier::new(Attribute::COLOR, color.expr());

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let lifetime = writer.lit(1.0).uniform(writer.lit(2.0)).expr(); // adjust for fade duration
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let size = writer.lit(0.05).expr();
    let init_size = SetAttributeModifier::new(Attribute::SIZE, size);

    let pos = writer.lit(Vec3::ZERO).expr();
    let init_pos = SetAttributeModifier::new(Attribute::POSITION, pos);

    let vel = writer.lit(Vec3::ZERO).expr();
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel);

    // update
    let update_accel = AccelModifier::new(writer.lit(Vec3::Y * -1.0).expr());

    // render
    let mut module = writer.finish();

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::ONE);
    gradient.add_key(0.5, Vec4::ONE);
    gradient.add_key(1.0, Vec4::ONE.with_w(0.0));

    let color_over_lifetime = ColorOverLifetimeModifier {
        gradient,
        blend: ColorBlendMode::Modulate,
        mask: ColorBlendMask::RGBA,
    };

    let round = RoundModifier::ellipse(&mut module);
    let orientation = OrientModifier {
        mode: OrientMode::ParallelCameraDepthPlane,
        ..default()
    };

    EffectAsset::new(1, SpawnerSettings::once(1.0.into()), module)
        .with_name("bullet_impact")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_color)
        .init(init_size)
        .update(update_accel)
        .render(color_over_lifetime)
        .render(orientation)
        .render(round)
}
