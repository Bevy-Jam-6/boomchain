pub(crate) mod assets;
pub(crate) mod effects;

use avian3d::prelude::*;
use bevy::{ecs::system::SystemParam, prelude::*};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::{
    auto_timer::{AutoTimer, OnAutoTimerFinish},
    despawn_after::Despawn,
    gameplay::{
        health::{Health, OnDamage, OnDeath},
        player::Player,
    },
    third_party::avian3d::CollisionLayer,
};

pub const EXPLOSION_PLAYER_DAMAGE_SCALE: f32 = 0.1;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((assets::plugin, effects::plugin));

    app.register_type::<(Explosive, ExplodeOnShoot, ExplodeOnContact)>();

    app.add_observer(on_shoot_explosive);
    app.add_observer(on_touch_explosive);
    app.add_observer(on_enemy_death);
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
pub(crate) struct Explosive {
    // TODO: Do we want to support a falloff?
    /// The radius of the explosion. Only entities within this radius will be affected.
    pub(crate) radius: f32,
    /// The strength of the explosion impulse.
    pub(crate) impulse_strength: f32,
    /// The damage dealt by the explosion.
    pub(crate) damage: f32,
    /// Whether the explosion damages the player.
    pub(crate) damages_player: bool,
}

impl Default for Explosive {
    fn default() -> Self {
        Self {
            radius: 3.5,
            impulse_strength: 25.0,
            damage: 100.0,
            damages_player: true,
        }
    }
}

/// An event that is triggered when an explosive should explode.
#[derive(Event, Clone, Copy, Debug, PartialEq)]
pub(crate) struct OnExplode;

/// A marker component for entities that have exploded or are in the process of exploding.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Reflect)]
pub(crate) struct Exploded;

/// A marker component for entities that should explode when interacted with.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[require(Explosive)]
pub(crate) struct ExplodeOnShoot {
    /// The maximum distance the explosion can be triggered from.
    pub(crate) max_distance: f32,
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
pub(crate) struct ExplodeOnContact {
    /// A [`LayerMask`] determining which [`CollisionLayers`] can trigger the explosion on contact.
    pub(crate) layers: LayerMask,
}

impl Default for ExplodeOnContact {
    fn default() -> Self {
        Self {
            layers: LayerMask::from(CollisionLayer::Character),
        }
    }
}

/// A marker component for entities that should explode on death.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[require(Explosive)]
pub(crate) struct ExplodeOnDeath;

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
fn on_enemy_death(
    trigger: Trigger<OnDeath>,
    mut commands: Commands,
    explosive_query: Query<(&GlobalTransform, &Explosive), With<ExplodeOnDeath>>,
) {
    let entity = trigger.target();

    // Get the explosive properties and transform of the entity.
    if let Ok((transform, explosive)) = explosive_query.get(entity) {
        // Trigger the explosion. We use a separate entity with a timer
        // to delay the explosion until the dismembered body parts of enemies
        // are ready for physics.
        // Hacky, but this is a game jam :D could be cleaned up though
        commands
            .spawn((
                RigidBody::Static,
                AutoTimer(Timer::from_seconds(0.1, TimerMode::Once)),
                // Just copy the transform and explosive properties to the temporary entity.
                transform.compute_transform(),
                *explosive,
            ))
            .observe(
                |trigger: Trigger<OnAutoTimerFinish>, mut commands: Commands| {
                    commands
                        .entity(trigger.target())
                        .trigger(OnExplode)
                        .despawn();
                },
            );
    }
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
    explosion_helper.commands.entity(entity).insert(Despawn);
}

/// A [`SystemParam`] for applying explosions in the world.
#[derive(SystemParam)]
pub(crate) struct ExplosionHelper<'w, 's> {
    collider_query: Query<'w, 's, (&'static Collider, &'static GlobalTransform)>,
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
    damageable_query: Query<'w, 's, Has<Player>, With<Health>>,
    spatial_query: SpatialQuery<'w, 's>,
    commands: Commands<'w, 's>,
}

impl ExplosionHelper<'_, '_> {
    /// Applies an explosion to all entities within the explosion radius at the given point.
    ///
    /// This also triggers the [`OnExplode`] event for any explosive entities hit by the explosion.
    pub(crate) fn apply_explosion(&mut self, explosive: &Explosive, point: Vec3) {
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
            // Get the body's transform, velocity, center of mass, and attached colliders.
            let Ok((rb, transform, mut lin_vel, mut ang_vel, local_com, colliders)) =
                self.body_query.get_mut(body)
            else {
                continue;
            };
            let global_com = transform.translation() + transform.rotation() * local_com.0;

            // If the entity has health, we apply damage to it.
            if let Ok(is_player) = self.damageable_query.get(body) {
                let mut damage = explosive.damage;

                if is_player {
                    // If the explosive damages the player, we apply a scaled damage immediately.
                    if !explosive.damages_player {
                        continue;
                    }
                    damage *= EXPLOSION_PLAYER_DAMAGE_SCALE;
                    self.commands.entity(body).trigger(OnDamage(damage));
                } else {
                    // For damage against enemies or explosives, we use a small delay.
                    let delay = 0.2;
                    self.commands
                        .entity(body)
                        .try_insert_if_new(AutoTimer(Timer::from_seconds(delay, TimerMode::Once)))
                        .observe(
                            move |trigger: Trigger<OnAutoTimerFinish>, mut commands: Commands| {
                                commands.entity(trigger.target()).trigger(OnDamage(damage));
                            },
                        );
                }
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
