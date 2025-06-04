use avian3d::prelude::*;
use bevy::{prelude::*, time::Stopwatch};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::{
    PrePhysicsAppSystems,
    gameplay::{
        health::Health,
        npc::stats::NpcStats,
        player::{Player, camera_shake::OnTrauma},
    },
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Attacking>();
    app.register_type::<AttackPhase>();
    app.register_type::<AttackStopwatch>();
    app.add_systems(
        RunFixedMainLoop,
        update_attack_phase.in_set(PrePhysicsAppSystems::AdvanceEnemyAttack),
    );
    app.add_observer(start_attack);
    app.add_observer(stop_attack);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn start_attack(trigger: Trigger<OnAdd, Attacking>, mut commands: Commands) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .insert((AttackPhase::Windup, AttackStopwatch::default()));
}

#[cfg_attr(feature = "hot_patch", hot)]
fn stop_attack(trigger: Trigger<OnRemove, Attacking>, mut commands: Commands) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .try_remove::<(AttackPhase, AttackStopwatch)>();
}

#[cfg_attr(feature = "hot_patch", hot)]
fn update_attack_phase(
    mut query: Query<(
        Entity,
        &mut AttackPhase,
        &Attacking,
        &mut AttackStopwatch,
        &NpcStats,
    )>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut phase, attacking, mut stopwatch, stats) in query.iter_mut() {
        stopwatch.0.tick(time.delta());
        match *phase {
            AttackPhase::Windup => {
                if stopwatch.0.elapsed_secs() > attacking.attack_start_secs() {
                    *phase = AttackPhase::Hit;
                    commands
                        .spawn((
                            HitboxOf(entity),
                            ChildOf(entity),
                            Sensor,
                            Transform::from_xyz(0.0, 0.0, -1.5),
                            Collider::cuboid(1.5 * stats.size, 1.5 * stats.size, 1.5 * stats.size),
                            CollisionEventsEnabled,
                            CollisionLayers::new(CollisionLayer::Sensor, CollisionLayer::Player),
                        ))
                        .observe(hit_player);
                }
            }
            AttackPhase::Hit => {
                if stopwatch.0.elapsed_secs() > attacking.attack_end_secs() {
                    *phase = AttackPhase::FollowThrough;
                    commands.entity(entity).queue_handled(
                        |mut entity: EntityWorldMut| {
                            entity.despawn_related::<Hitbox>();
                        },
                        bevy::ecs::error::ignore,
                    );
                }
            }
            AttackPhase::FollowThrough => {}
        }
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn hit_player(
    trigger: Trigger<OnCollisionStart>,
    mut player: Query<&mut Health, With<Player>>,
    name: Query<NameOrEntity>,
    mut commands: Commands,
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
    commands.trigger(OnTrauma(0.7));
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
    pub(crate) speed: f32,
    pub(crate) damage: f32,
    pub(crate) dir: Option<Dir3>,
}

#[derive(Component, Debug, Reflect, Default, Clone)]
#[reflect(Component)]
struct AttackStopwatch(Stopwatch);

#[derive(Debug, Reflect, Component, Default, Clone, Copy)]
#[reflect(Component)]
enum AttackPhase {
    #[default]
    Windup,
    Hit,
    FollowThrough,
}

impl Attacking {
    pub(crate) fn attack_start_secs(&self) -> f32 {
        self.frame_to_secs(24)
    }

    pub(crate) fn attack_end_secs(&self) -> f32 {
        self.frame_to_secs(29)
    }

    fn frame_to_secs(&self, frame: u32) -> f32 {
        let fps = 24;
        let time = frame as f32 / fps as f32;
        time / self.speed
    }
}
