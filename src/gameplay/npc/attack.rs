use avian3d::prelude::*;
use bevy::{ecs::spawn::SpawnWith, prelude::*, state::commands};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::third_party::avian3d::CollisionLayer;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Attacking>();
    app.add_observer(start_attack);
    app.add_observer(stop_attack);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn start_attack(trigger: Trigger<OnAdd, Attacking>, mut commands: Commands) {
    info!("start_attack");
    let entity = trigger.target();
    commands
        .spawn((
            HitboxOf(entity),
            ChildOf(entity),
            Sensor,
            Transform::from_xyz(0.0, 0.0, -0.5),
            Collider::cuboid(1.5, 1.5, 1.5),
            CollisionEventsEnabled,
            CollisionLayers::new(CollisionLayer::Sensor, CollisionLayer::Player),
        ))
        .observe(hit_player);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn stop_attack(trigger: Trigger<OnRemove, Attacking>, mut commands: Commands) {
    info!("stop_attack");
    let entity = trigger.target();
    commands.entity(entity).despawn_related::<Hitbox>();
}

fn hit_player(trigger: Trigger<OnCollisionStart>, mut commands: Commands) {
    info!("hit_player");
}

#[derive(Component, Deref, DerefMut, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Hitbox)]
pub(crate) struct HitboxOf(Entity);

#[derive(Component, Deref, DerefMut, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = HitboxOf)]
pub(crate) struct Hitbox(Entity);

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Attacking {
    pub(crate) dir: Option<Dir3>,
}
