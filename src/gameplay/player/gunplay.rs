use super::{Player, camera::PlayerCamera, default_input::Shoot};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::events::Started;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Shooting;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shooting);
    app.add_observer(print_hits);
}

fn shooting(trigger: Trigger<Started<Shoot>>, mut commands: Commands) {
    let entity = trigger.target();
    commands.entity(entity).insert(Shooting);
}

fn print_hits(
    _trigger: Trigger<Started<Shoot>>,
    spatial_query: SpatialQuery,
    player_camera_parent: Single<&Transform, With<PlayerCamera>>,
    name: Query<NameOrEntity>,
    player: Single<Entity, With<Player>>,
) {
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

    if hits.len() == 0 {
        // Sorry Joona, had to bring in some swiss german ;).
        info!("denäbe!")
    }
}
