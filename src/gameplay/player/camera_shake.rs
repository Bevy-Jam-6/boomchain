use bevy::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use noiz::{
    prelude::{common_noise::Perlin, *},
    rng::NoiseRng,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CameraShake>();
    app.register_type::<NonTraumaTransform>();

    app.add_systems(
        PostUpdate,
        set_final_camera_transform.before(TransformSystem::TransformPropagate),
    );
    app.add_systems(PreUpdate, decay_trauma);
    app.add_observer(on_trauma);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct CameraShake {
    trauma: f32,
    /// Per second
    trauma_decay: f32,
    yaw_seed: NoiseRng,
    pitch_seed: NoiseRng,
    roll_seed: NoiseRng,
    noise: Perlin,
}

impl Default for CameraShake {
    fn default() -> Self {
        Self {
            trauma: 0.0,
            trauma_decay: 0.5,
            yaw_seed: NoiseRng(1),
            pitch_seed: NoiseRng(2),
            roll_seed: NoiseRng(3),
            noise: Perlin::default(),
        }
    }
}

#[derive(Component, Default, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub(crate) struct NonTraumaTransform(pub(crate) Transform);

#[derive(Event)]
pub(crate) struct OnTrauma(pub(crate) f32);

#[cfg_attr(feature = "hot_patch", hot)]
fn on_trauma(trigger: Trigger<OnTrauma>, mut camera_shake: Single<&mut CameraShake>) {
    camera_shake.trauma = (camera_shake.trauma + trigger.event().0).clamp(0.0, 1.0);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn decay_trauma(mut camera_shake: Single<&mut CameraShake>, time: Res<Time>) {
    camera_shake.trauma =
        (camera_shake.trauma - camera_shake.trauma_decay * time.delta_secs()).max(0.0);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn set_final_camera_transform(
    camera_shake: Single<(&mut Transform, &NonTraumaTransform, &mut CameraShake)>,
    time: Res<Time>,
) {
    let (mut transform, non_trauma_transform, mut camera_shake) = camera_shake.into_inner();
    let shake = camera_shake.trauma * camera_shake.trauma;
    let max_yaw = 50.0_f32.to_radians();
    let max_pitch = 50.0_f32.to_radians();
    let max_roll = 10.0_f32.to_radians();

    let noise = camera_shake.noise;
    let speed = 4.0;
    let noise_input = Vec2::new(time.elapsed_secs() * speed, 0.0);

    let yaw = max_yaw * shake * noise.evaluate(noise_input, &mut camera_shake.yaw_seed);
    let pitch = max_pitch * shake * noise.evaluate(noise_input, &mut camera_shake.pitch_seed);
    let roll = max_roll * shake * noise.evaluate(noise_input, &mut camera_shake.roll_seed);

    let shake_rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    *transform = **non_trauma_transform;
    transform.rotate(shake_rotation);

    info!("trauma: {}", camera_shake.trauma);
}
