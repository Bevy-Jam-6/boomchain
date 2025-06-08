use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};

#[cfg(not(feature = "native"))]
use crate::gameplay::player::camera::PlayerCamera;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_directional_light);
    #[cfg(not(feature = "native"))]
    app.add_systems(Update, disable_weird_light);
}

fn setup_directional_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 5_000.0,
            color: Color::srgb_u8(200, 190, 255),
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            #[cfg(not(feature = "native"))]
            // Fix bug on WebGPU :/
            num_cascades: 1,
            maximum_distance: 400.0,
            first_cascade_far_bound: 40.0,
            ..default()
        }
        .build(),
        Transform::default().looking_to(Vec3::new(-1.75, -1.0, 0.5), Vec3::Y),
    ));
}

#[cfg(not(feature = "native"))]
// Fix bug on WebGPU :/
fn disable_weird_light(
    mut lights: Query<(&Transform, &mut DirectionalLight)>,
    player: Single<&Transform, With<PlayerCamera>>,
) {
    for (light_transform, mut light) in lights.iter_mut() {
        let dot = light_transform.forward().dot(player.forward().into());
        info!("dot: {}", dot);
        if dot > 0.8 {
            light.illuminance = 0.0;
        } else {
            light.illuminance = 5_000.0;
        }
    }
}
