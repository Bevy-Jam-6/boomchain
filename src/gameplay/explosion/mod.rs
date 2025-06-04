mod assets;
pub(crate) mod effects;

use avian3d::prelude::*;
use bevy::{
    ecs::{error::ignore, system::SystemParam},
    prelude::*,
};
use bevy_enhanced_input::events::Started;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use super::player::{Player, camera::PlayerCamera, default_input::Shoot};
use crate::{
    auto_timer::{AutoTimer, OnAutoTimerFinish},
    gameplay::{
        health::{Health, OnDeath},
        npc::Npc,
    },
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, effects::plugin));

    app.register_type::<(Explosive, ExplodeOnShoot, ExplodeOnContact)>();

    app.add_observer(on_shoot_explosive);
    app.add_observer(on_touch_explosive);
    app.add_observer(on_explode);

    // Insert `CollisionEventsEnabled` for all entities that can explode on contact,
    // and their child colliders.
    app.add_observer(
        |trigger: Trigger<OnAdd, ExplodeOnContact>,
         mut commands: Commands,
         children: Query<&Children>,
         colliders: Query<(), With<Collider>>| {
            commands
                .entity(trigger.target())
                .insert(CollisionEventsEnabled);
            for child in children.iter_descendants(trigger.target()) {
                if colliders.contains(child) {
                    commands.entity(child).insert(CollisionEventsEnabled);
                }
            }
        },
    );
}

/// A component for making an entity an explosive.
///
/// The explosion can be activated by triggering the [`OnExplode`] event.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Explosive {
    // TODO: Do we want to support a falloff?
    /// The radius of the explosion. Only entities within this radius will be affected.
    pub radius: f32,
    /// The strength of the explosion impulse.
    pub impulse_strength: f32,
    /// The damage dealt by the explosion.
    pub damage: f32,
}

impl Default for Explosive {
    fn default() -> Self {
        Self {
            radius: 3.0,
            impulse_strength: 15.0,
            damage: 100.0,
        }
    }
}

/// An event that is triggered when an explosive should explode.
#[derive(Event, Clone, Copy, Debug, PartialEq)]
pub struct OnExplode;

/// A marker component for entities that have exploded or are in the process of exploding.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub struct Exploded;

/// A marker component for entities that should explode when interacted with.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[require(Explosive)]
pub struct ExplodeOnShoot {
    /// The maximum distance the explosion can be triggered from.
    pub max_distance: f32,
}

impl Default for ExplodeOnShoot {
    fn default() -> Self {
        Self {
            max_distance: f32::MAX,
        }
    }
}

/// A marker component for entities that should explode on contact.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[require(Explosive)]
pub struct ExplodeOnContact {
    /// A [`LayerMask`] determining which [`CollisionLayers`] can trigger the explosion on contact.
    pub layers: LayerMask,
}

impl Default for ExplodeOnContact {
    fn default() -> Self {
        Self {
            layers: LayerMask::from(CollisionLayer::Character),
        }
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn on_shoot_explosive(
    trigger: Trigger<OnDeath>,
    explosive_query: Query<&ExplodeOnShoot>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    if let Ok(_explosive) = explosive_query.get(entity) {
        // Trigger the explosion.
        commands.entity(entity).trigger(OnExplode);
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn on_touch_explosive(
    trigger: Trigger<OnCollisionStart>,
    collider_of_query: Query<&ColliderOf>,
    explosive_query: Query<&ExplodeOnContact>,
    layer_query: Query<&CollisionLayers>,
    mut commands: Commands,
) {
    let Ok(&ColliderOf { body }) = collider_of_query.get(trigger.target()) else {
        return;
    };

    let Ok(explosive) = explosive_query.get(body) else {
        return;
    };

    let Ok(touching_layers) = layer_query.get(trigger.collider) else {
        return;
    };

    // Check if the touching entity is in the layers that can trigger the explosion.
    let explosive_layers = CollisionLayers::new(LayerMask::ALL, explosive.layers);
    if !explosive_layers.interacts_with(*touching_layers) {
        return;
    }

    // Trigger the explosion.
    commands.entity(body).trigger(OnExplode);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn on_explode(
    trigger: Trigger<OnExplode>,
    query: Query<(&Explosive, &GlobalTransform, &ComputedCenterOfMass), Without<Exploded>>,
    mut explosion_helper: ExplosionHelper,
) {
    let entity = trigger.target();

    // Get the explosive properties and global center of mass.
    let Ok((explosive, explosive_transform, local_com)) = query.get(entity) else {
        return;
    };
    let explosive_rotation = explosive_transform.rotation();
    let explosive_global_com = explosive_transform.translation() + explosive_rotation * local_com.0;

    // Mark the explosive as exploded. This prevents the explosion
    // from being triggered multiple times in the case of chain reactions.
    //
    // If we do want to support re-exploding, we could remove this component
    // at the end of the frame.
    //
    // NOTE: We must use the command queue from the `ExplosionHelper` here
    //       to get the correct ordering and avoid infinite recursion.
    //       Using `Commands` that are local to this system breaks things :D
    //       (It took me an hour to figure this out...)
    explosion_helper
        .commands
        .entity(entity)
        .try_insert(Exploded);

    // Apply the explosion at the center of mass of the explosive.
    explosion_helper.apply_explosion(explosive, explosive_global_com);

    // Despawn the explosive entity after the explosion.
    explosion_helper.commands.entity(entity).try_despawn();
}

/// A [`SystemParam`] for applying explosions in the world.
#[derive(SystemParam)]
pub struct ExplosionHelper<'w, 's> {
    collider_query: Query<'w, 's, (&'static Collider, &'static GlobalTransform)>,
    explosive_query: Query<'w, 's, (), (With<Explosive>, Without<Exploded>)>,
    collider_of_query: Query<'w, 's, &'static ColliderOf>,
    body_query: Query<
        'w,
        's,
        (
            &'static RigidBody,
            &'static GlobalTransform,
            &'static mut LinearVelocity,
            &'static mut AngularVelocity,
            &'static ComputedCenterOfMass,
            &'static RigidBodyColliders,
        ),
    >,
    npc_health_query: Query<'w, 's, &'static mut Health, With<Npc>>,
    spatial_query: SpatialQuery<'w, 's>,
    commands: Commands<'w, 's>,
}

impl ExplosionHelper<'_, '_> {
    /// Applies an explosion to all entities within the explosion radius at the given point.
    ///
    /// This also triggers the [`OnExplode`] event for any explosive entities hit by the explosion.
    pub fn apply_explosion(&mut self, explosive: &Explosive, point: Vec3) {
        // Query for all collider entities of characters and props within the explosion radius.
        let shape = Collider::sphere(explosive.radius);
        let filter = SpatialQueryFilter::default();
        let hit_entities =
            self.spatial_query
                .shape_intersections(&shape, point, Quat::IDENTITY, &filter);

        // Get the bodies of the hit entities.
        let mut hit_bodies = self
            .collider_of_query
            .iter_many(hit_entities)
            .map(|&ColliderOf { body }| body)
            .collect::<Vec<Entity>>();

        // Deduplicate the hit bodies to avoid applying multiple impulses to the same body
        // in case of multiple colliders.
        hit_bodies.sort();
        hit_bodies.dedup();

        // Apply the explosion impulse to each hit body.
        for body in hit_bodies {
            if let Ok(mut health) = self.npc_health_query.get_mut(body) {
                // If the body is an NPC, apply damage.
                health.damage(explosive.damage);
            }

            // Get the body's transform, velocity, center of mass, and attached colliders.
            let Ok((rb, transform, mut lin_vel, mut ang_vel, local_com, colliders)) =
                self.body_query.get_mut(body)
            else {
                continue;
            };
            let global_com = transform.translation() + transform.rotation() * local_com.0;

            // If the entity is also an explosive, trigger its explosion.
            // A chain reaction of props going boom!
            if self.explosive_query.contains(body) {
                // Delay the explosion slightly based on distance.
                let delay = point.distance(global_com) * 0.05;
                self.commands
                    .entity(body)
                    .try_insert_if_new(AutoTimer(Timer::from_seconds(delay, TimerMode::Once)))
                    .observe(
                        |trigger: Trigger<OnAutoTimerFinish>, mut commands: Commands| {
                            // We need to use `queue_handled` in case the explosion was triggered twice
                            // and the entity was already despawned.
                            commands.entity(trigger.target()).queue_handled(
                                |mut entity_mut: EntityWorldMut| {
                                    entity_mut.trigger(OnExplode);
                                },
                                ignore,
                            );
                        },
                    );
            }

            if !rb.is_dynamic() {
                continue;
            }

            // Iterate over the colliders of the body to find the point closest
            // to the source of the explosion.
            let (closest_point, _is_inside) = self
                .collider_query
                .iter_many(colliders.iter())
                .map(|(collider, transform)| {
                    collider.project_point(
                        transform.translation(),
                        transform.rotation(),
                        point,
                        true,
                    )
                })
                .min_by(|(a, _), (b, _)| {
                    a.distance_squared(point)
                        .partial_cmp(&b.distance_squared(point))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .expect("Body hit by explosion has no colliders. Huh???");

            // Compute the impulse direction and magnitude.
            // We ignore mass properties here to make explosions more predictable and fun.
            // TODO: We could support a falloff based on the distance from the center of the explosion.
            let explosion_direction = (closest_point - point).normalize_or_zero();
            let lin_impulse = explosive.impulse_strength * explosion_direction;
            let ang_impulse = (closest_point - global_com).cross(lin_impulse);

            // Apply the impulses to the body's velocities.
            lin_vel.0 += lin_impulse;
            ang_vel.0 += ang_impulse;
        }
    }
}
