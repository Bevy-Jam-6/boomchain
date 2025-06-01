use super::{Player, animation::PlayerAnimations, camera::PlayerCamera, default_input::Shoot};
use crate::gameplay::{animation::AnimationPlayers, crosshair::CrosshairState};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::events::Started;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shooting);
    app.add_observer(print_hits);
}

fn shooting(
    _trigger: Trigger<Started<Shoot>>,
    mut query: Query<&AnimationPlayers>,
    mut q_animation: Query<(
        &PlayerAnimations,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    crosshair_state: Single<&CrosshairState>,
) {
    for anim_players in &mut query {
        let mut iter = q_animation.iter_many_mut(anim_players.iter());
        while let Some((animations, mut anim_player, mut transitions)) = iter.fetch_next() {
            if crosshair_state.wants_invisible.is_empty() {
                transitions.play(
                    &mut anim_player,
                    animations.shooting,
                    Duration::from_millis(150),
                );
            }
        }
    }
}

fn print_hits(
    _trigger: Trigger<Started<Shoot>>,
    spatial_query: SpatialQuery,
    player_camera_parent: Single<&Transform, With<PlayerCamera>>,
    name: Query<NameOrEntity>,
    player: Single<Entity, With<Player>>,
) {
    info!("pew!");

    // Ray origin and direction
    let origin = player_camera_parent.translation;
    let direction = player_camera_parent.forward();

    // Configuration for the ray cast
    let max_distance = 100.0;
    let solid = true;
    let filter = SpatialQueryFilter::default().with_excluded_entities([*player]);

    // Cast ray and print first hit
    if let Some(first_hit) = spatial_query.cast_ray(origin, direction, max_distance, solid, &filter)
    {
        let name = name.get(first_hit.entity).unwrap();
        info!("First hit: {name}");
    }

    // Cast ray and get up to 20 hits
    let hits = spatial_query.ray_hits(origin, direction, max_distance, 20, solid, &filter);

    // Print hits
    for hit in hits.iter() {
        let name = name.get(hit.entity).unwrap();
        info!("Hit: {name}");
    }
}
