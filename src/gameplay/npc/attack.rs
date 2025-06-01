use avian3d::prelude::*;
use bevy::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::{
    gameplay::{health::Health, player::Player},
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Attacking>();
    app.add_observer(start_attack);
    app.add_observer(stop_attack);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn start_attack(trigger: Trigger<OnAdd, Attacking>, mut commands: Commands) {
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
    let entity = trigger.target();
    commands.entity(entity).despawn_related::<Hitbox>();
}

fn hit_player(
    trigger: Trigger<OnCollisionStart>,
    mut player: Query<&mut Health, With<Player>>,
    name: Query<NameOrEntity>,
) {
    let Some(body) = trigger.event().body else {
        error!("Enemy hit collision without body");
        return;
    };
    let Ok(mut health) = player.get_mut(body) else {
        let name = name.get(body).unwrap();
        error!("Enemy hit non-player: {name}");
        return;
    };
    health.damage(10.0);
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
