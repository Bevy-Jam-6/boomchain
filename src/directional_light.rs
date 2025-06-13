use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};

#[cfg(not(feature = "native"))]
use crate::gameplay::player::camera::PlayerCamera;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_directional_light);
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
            maximum_distance: 400.0,
            first_cascade_far_bound: 40.0,
            ..default()
        }
        .build(),
        Transform::default().looking_to(Vec3::new(-1.75, -1.0, 0.5), Vec3::Y),
    ));
}
