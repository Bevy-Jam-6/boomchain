use bevy::{
    audio::{SpatialScale, Volume},
    prelude::*,
};
use bevy_firework::{
    bevy_utilitarian::prelude::RandF32,
    core::{BlendMode, ParticleSpawner, ParticleSpawnerFinished, SpawnTransformMode},
    curve::FireworkGradient,
    emission_shape::EmissionShape,
};
use bevy_hanabi::{
    AccelModifier, Attribute, ColorBlendMask, ColorBlendMode, ColorOverLifetimeModifier,
    EffectAsset, ExprWriter, Gradient, LinearDragModifier, ParticleEffect, ScalarType,
    SetAttributeModifier, SetPositionSphereModifier, SetVelocitySphereModifier, ShapeDimension,
    SpawnerSettings,
};

use super::{OnExplode, assets::ExplosionAssets};
use crate::{audio::SoundEffect, platform_support::is_webgpu_or_native};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PropExplosionVfx>();

    app.add_observer(on_explode_prop);
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct PropExplosionVfx;

fn on_explode_prop(
    trigger: Trigger<OnExplode>,
    query: Query<&GlobalTransform, With<PropExplosionVfx>>,
    mut explosion_assets: ResMut<ExplosionAssets>,
    mut commands: Commands,
) {
    let Ok(transform) = query.get(trigger.target()) else {
        return;
    };

    let rng = &mut rand::thread_rng();
    let sound_effect = explosion_assets.prop_explosion_sfx.pick(rng).clone();

    let entity = commands
        .spawn((
            transform.compute_transform(),
            AudioPlayer(sound_effect),
            PlaybackSettings::DESPAWN
                .with_spatial(true)
                .with_speed(0.9)
                .with_volume(Volume::Linear(3.5))
                .with_spatial_scale(SpatialScale::new(1.0 / 10.0)),
            SoundEffect,
        ))
        .id();

    // Use Hanabi if supported, otherwise use `bevy_firework` as a fallback.
    if is_webgpu_or_native() {
        commands.entity(entity).insert(ParticleEffect::new(
            explosion_assets.prop_explosion_vfx.clone(),
        ));
    } else {
        commands
            .spawn((
                bevy_firework_prop_explosion(),
                Transform::from_translation(transform.translation() + Vec3::Y),
            ))
            .observe(
                |trigger: Trigger<ParticleSpawnerFinished>, mut commands: Commands| {
                    commands.entity(trigger.target()).despawn();
                },
            );
    }
}

fn bevy_firework_prop_explosion() -> ParticleSpawner {
    let gradient = FireworkGradient::uneven_samples(vec![
        (0.0, LinearRgba::new(1.0, 1.0, 0.8, 1.0)),
        (0.2, LinearRgba::new(1.0, 0.8, 0.3, 1.0)),
        (0.6, LinearRgba::new(1.0, 0.3, 0.1, 0.8)),
        (1.0, LinearRgba::new(1.0, 0.2, 0.1, 0.0)),
    ]);

    ParticleSpawner {
        one_shot: true,
        rate: 600.0,
        emission_shape: EmissionShape::Sphere(2.0),
        lifetime: RandF32 { min: 0.2, max: 1.0 },
        inherit_parent_velocity: true,
        initial_velocity_radial: RandF32 {
            min: 0.0,
            max: 10.0,
        },
        initial_scale: RandF32 {
            min: 0.01,
            max: 0.1,
        },
        color: gradient.clone(),
        blend_mode: BlendMode::Blend,
        linear_drag: 7.0,
        fade_edge: 0.4,
        pbr: false,
        acceleration: Vec3::new(0., -8.0, 0.),
        spawn_transform_mode: SpawnTransformMode::Local,
        ..default()
    }
}

pub(super) fn hanabi_prop_explosion(world: &mut World) -> EffectAsset {
    let unit_sphere: Handle<Mesh> = world.add_asset(Sphere::new(0.5).mesh().ico(4).unwrap());

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 1.0, 0.8, 1.0));
    gradient.add_key(0.2, Vec4::new(1.0, 0.8, 0.3, 1.0));
    gradient.add_key(0.6, Vec4::new(1.0, 0.3, 0.1, 0.8));
    gradient.add_key(1.0, Vec4::new(1.0, 0.2, 0.1, 0.0));

    let writer = ExprWriter::new();

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(2.).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Also initialize a radial initial velocity to 6 units/sec
    // away from the (same) sphere center.
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(10.0)).expr(),
    };

    let init_size = SetAttributeModifier::new(
        Attribute::SIZE,
        (writer.rand(ScalarType::Float) * writer.lit(0.1) + writer.lit(0.01)).expr(),
    );

    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles will stay
    // alive forever, and new particles can't be spawned instead.
    let lifetime = (writer.rand(ScalarType::Float) * writer.lit(0.8) + writer.lit(0.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let mut module = writer.finish();

    // Add drag.
    let drag = module.lit(7.0);
    let update_drag = LinearDragModifier::new(drag);

    // Every frame, add a gravity-like acceleration downward
    let accel = module.lit(Vec3::new(0.0, -8.0, 0.0));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset
    EffectAsset::new(10_000, SpawnerSettings::once(600.0.into()), module)
        .with_name("PropExplosionEffect")
        .init(init_pos)
        .init(init_vel)
        .init(init_size)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        // Render the particles with a color gradient over their
        // lifetime. This maps the gradient key 0 to the particle spawn
        // time, and the gradient key 1 to the particle death (10s).8
        .render(ColorOverLifetimeModifier {
            gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .mesh(unit_sphere)
}
