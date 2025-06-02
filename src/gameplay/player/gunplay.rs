use crate::{
    audio::sound_effect,
    gameplay::{
        health::{Death, Health},
        npc::Npc,
    },
    third_party::avian3d::CollisionLayer,
};

use super::{assets::PlayerAssets, camera::PlayerCamera, default_input::Shoot};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::events::Started;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Shooting;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shooting);
    app.add_observer(shooting_sounds);
    app.add_observer(handle_hits);
    app.add_observer(on_death);
}

fn shooting(trigger: Trigger<Started<Shoot>>, mut commands: Commands) {
    let entity = trigger.target();
    commands.entity(entity).insert(Shooting);
}

fn shooting_sounds(
    _trigger: Trigger<Started<Shoot>>,
    mut commands: Commands,
    mut player_assets: ResMut<PlayerAssets>,
) {
    let rng = &mut rand::thread_rng();
    let shooting_sound = player_assets.shooting_sounds.pick(rng).clone();

    commands.spawn(sound_effect(shooting_sound));
}

fn handle_hits(
    _trigger: Trigger<Started<Shoot>>,
    spatial_query: SpatialQuery,
    player_camera_parent: Single<&Transform, With<PlayerCamera>>,
    mut npcs: Query<&mut Health, With<Npc>>,
) {
    // Ray origin and direction
    let origin = player_camera_parent.translation;
    let direction = player_camera_parent.forward();

    // Configuration for the ray cast
    let max_distance = 100.0;
    let solid = true;
    let filter =
        SpatialQueryFilter::default().with_mask([CollisionLayer::Npc, CollisionLayer::Prop]);

    // Cast ray and print first hit
    let Some(first_hit) = spatial_query.cast_ray(origin, direction, max_distance, solid, &filter)
    else {
        return;
    };
    let Ok(mut health) = npcs.get_mut(first_hit.entity) else {
        return;
    };

    let gun_damage = 10.0;
    health.damage(gun_damage);
}

fn on_death(trigger: Trigger<Death>, name: Query<NameOrEntity>) {
    let entity = trigger.target();
    let name = name.get(entity).unwrap();
    info!("Just died: {name}");
}
