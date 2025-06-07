use std::time::Duration;

use bevy::{
    audio::{SpatialScale, Volume},
    color::palettes::css::ORANGE,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_firework::{
    bevy_utilitarian::prelude::RandF32,
    core::{BlendMode, ParticleSpawner, SpawnTransformMode},
    curve::FireworkGradient,
    emission_shape::EmissionShape,
};
use bevy_hanabi::{
    AccelModifier, Attribute, ColorBlendMask, ColorBlendMode, ColorOverLifetimeModifier,
    EffectAsset, EffectProperties, ExprWriter, Gradient, LinearDragModifier, ParticleEffect,
    ScalarType, ScalarValue, SetAttributeModifier, SetPositionSphereModifier,
    SetVelocitySphereModifier, ShapeDimension, SpawnerSettings, Value,
};
use bevy_mesh_decal::spray_decal;
use rand::Rng as _;

use super::{OnExplode, assets::ExplosionAssets};
use crate::{
    RenderLayer,
    audio::SoundEffect,
    despawn_after::DespawnAfter,
    gameplay::{
        explosion::ExplodeOnDeath,
        gore_settings::{Gore, GoreSettings},
        health::OnDeath,
        npc::stats::NpcStats,
    },
    platform_support::is_webgpu_or_native,
};

const EXPLOSION_LIGHT_INTENSITY: f32 = 50_000.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PropExplosionVfx>();

    app.add_observer(on_explode_prop);
    app.add_observer(on_enemy_death);
    app.add_systems(Update, fade_out_despawned_point_light);
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Debug, Default)]
pub(crate) struct PropExplosionVfx;

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

    commands.spawn((
        transform.compute_transform(),
        AudioPlayer(sound_effect),
        PlaybackSettings::DESPAWN
            .with_spatial(true)
            .with_speed(0.9)
            .with_volume(Volume::Linear(3.5))
            .with_spatial_scale(SpatialScale::new(1.0 / 10.0)),
        SoundEffect,
    ));

    // Use Hanabi if supported, otherwise use `bevy_firework` as a fallback.
    if is_webgpu_or_native() {
        commands.spawn((
            Transform::from_translation(transform.translation()),
            DespawnAfter::new(Duration::from_secs(1)),
            ParticleEffect::new(explosion_assets.prop_explosion_vfx.clone()),
            RenderLayers::from(RenderLayer::PARTICLES),
            children![PointLight {
                intensity: EXPLOSION_LIGHT_INTENSITY,
                range: 10.0,
                radius: 0.25,
                shadows_enabled: false,
                color: ORANGE.into(),
                ..default()
            }],
        ));
    } else {
        commands.spawn((
            bevy_firework_prop_explosion(),
            Transform::from_translation(transform.translation()),
            DespawnAfter::new(Duration::from_secs(1)),
            PointLight {
                intensity: EXPLOSION_LIGHT_INTENSITY,
                range: 10.0,
                radius: 0.25,
                shadows_enabled: false,
                color: ORANGE.into(),
                ..default()
            },
        ));
    }
}

fn on_enemy_death(
    trigger: Trigger<OnDeath>,
    query: Query<(&GlobalTransform, Option<&NpcStats>), With<ExplodeOnDeath>>,
    mut explosion_assets: ResMut<ExplosionAssets>,
    gore_settings: Res<GoreSettings>,
    mut commands: Commands,
) {
    let Ok((transform, stats)) = query.get(trigger.target()) else {
        return;
    };
    let transform = transform.compute_transform();
    let scale = stats.map_or(1.0, |s| s.size);

    // Spawn the explosion particle effect.
    let properties = EffectProperties::default().with_properties([(
        "scale".to_string(),
        Value::Scalar(ScalarValue::Float(scale)),
    )]);
    commands.spawn((
        ParticleEffect::new(explosion_assets.enemy_explosion_vfx.clone()),
        properties,
        transform,
        DespawnAfter::new(Duration::from_secs(1)),
        RenderLayers::from(RenderLayer::PARTICLES),
    ));

    if gore_settings.blood_decals == Gore::None {
        return;
    }

    // Spray some blood splatter decals.
    let blood_decal_texture = explosion_assets
        .blood_splatter
        .pick(&mut rand::thread_rng())
        .clone();

    // Generate a somewhat randomized rotation for the spray decal.
    let mut rng = rand::thread_rng();
    let mut rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    rotation = Quat::from_rotation_y(rng.gen_range(0.0..std::f32::consts::TAU)) * rotation;
    rotation = Quat::from_rotation_z(rng.gen_range(0.0..10f32.to_radians())) * rotation;
    let scale_x = rng.gen_range(0.8..1.2);
    let scale_y = rng.gen_range(0.8..1.2);
    let scale_z = rng.gen_range(0.8..1.2);
    let base_scale = scale * 2.5;

    let spray_transform = transform
        .with_translation(transform.translation - Vec3::Y * 0.3)
        .with_scale(Vec3::new(
            base_scale * scale_x,
            base_scale * scale_y,
            base_scale * scale_z,
        ))
        .with_rotation(rotation);
    spray_decal(&mut commands, blood_decal_texture, spray_transform);
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
        radius: writer.lit(2.0).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Initialize a radial initial velocity.
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(10.0)).expr(),
    };

    // Initialize the size of the particle.
    let init_size = SetAttributeModifier::new(
        Attribute::SIZE,
        (writer.rand(ScalarType::Float) * writer.lit(0.1) + writer.lit(0.01)).expr(),
    );

    // Initialize the total lifetime of the particle.
    let lifetime = (writer.rand(ScalarType::Float) * writer.lit(0.8) + writer.lit(0.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let mut module = writer.finish();

    // Add drag.
    let drag = module.lit(7.0);
    let update_drag = LinearDragModifier::new(drag);

    // Every frame, add a gravity-like acceleration downward.
    let accel = module.lit(Vec3::new(0.0, -8.0, 0.0));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset.
    EffectAsset::new(500_000, SpawnerSettings::once(5000.0.into()), module)
        .with_name("PropExplosionEffect")
        .init(init_pos)
        .init(init_vel)
        .init(init_size)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .mesh(unit_sphere)
}

pub(super) fn hanabi_enemy_explosion(world: &mut World) -> EffectAsset {
    // TODO: Use slightly deformed icospheres
    let unit_sphere: Handle<Mesh> = world.add_asset(Sphere::new(0.5).mesh().ico(4).unwrap());

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0));
    gradient.add_key(0.6, Vec4::new(0.8, 0.0, 0.0, 1.0));
    gradient.add_key(1.0, Vec4::new(0.6, 0.0, 0.0, 0.0));

    let writer = ExprWriter::new();
    let scale = writer.add_property("scale", ScalarValue::Float(1.0).into());

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 1 units.
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: (writer.prop(scale) * writer.lit(1.0)).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Initialize a radial initial velocity.
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.prop(scale) * writer.rand(ScalarType::Float) * writer.lit(5.0)).expr(),
    };

    // Initialize the size of the particle.
    let init_size = SetAttributeModifier::new(
        Attribute::SIZE,
        (writer.prop(scale)
            * (writer.rand(ScalarType::Float) * writer.lit(0.1) + writer.lit(0.01)))
        .expr(),
    );

    // Initialize the total lifetime of the particle.
    let lifetime = (writer.rand(ScalarType::Float) * writer.lit(1.5) + writer.lit(0.5)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let mut module = writer.finish();

    // Add drag.
    let drag = module.lit(3.0);
    let update_drag = LinearDragModifier::new(drag);

    // Every frame, add a gravity-like acceleration downward.
    let accel = module.lit(Vec3::new(0.0, -9.81, 0.0));
    let update_accel = AccelModifier::new(accel);

    // Create the effect asset.
    EffectAsset::new(500_000, SpawnerSettings::once(500.0.into()), module)
        .with_name("EnemyExplosionEffect")
        .init(init_pos)
        .init(init_vel)
        .init(init_size)
        .init(init_lifetime)
        .update(update_drag)
        .update(update_accel)
        .render(ColorOverLifetimeModifier {
            gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .mesh(unit_sphere)
}

fn fade_out_despawned_point_light(mut query: Query<(&mut PointLight, &DespawnAfter)>) {
    for (mut light, despawn_timer) in query.iter_mut() {
        light.intensity = EXPLOSION_LIGHT_INTENSITY.lerp(0.0, despawn_timer.0.fraction());
    }
}
