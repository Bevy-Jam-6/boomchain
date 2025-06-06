use avian3d::prelude::LinearVelocity;
use bevy::{ecs::relationship::Relationship, prelude::*};
use bevy_landmass::{AgentDesiredVelocity3d, AgentState};
use bevy_tnua::prelude::TnuaController;

use crate::{
    PostPhysicsAppSystems,
    audio::sound_effect,
    despawn_after::Despawn,
    gameplay::{
        health::{Health, OnDamage},
        npc::{ai_state::AiState, navigation::AgentOf},
        player::{Player, assets::PlayerAssets, camera_shake::OnTrauma},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shake_on_hit);
    app.add_systems(
        Update,
        (tag_out_of_navmesh, despawn_out_of_navmesh).in_set(PostPhysicsAppSystems::TickTimers),
    );
}

fn shake_on_hit(
    trigger: Trigger<OnDamage>,
    player: Query<&Health, With<Player>>,
    mut commands: Commands,
    mut player_assets: ResMut<PlayerAssets>,
) {
    let Ok(health) = player.get(trigger.target()) else {
        return;
    };

    let base_trauma = 0.7 / 10.0;
    let dmg = trigger.event().0;
    commands.trigger(OnTrauma(base_trauma * dmg));

    if !health.is_dead() {
        let handle = player_assets
            .hurt_sounds
            .pick(&mut rand::thread_rng())
            .clone();
        commands.spawn(sound_effect(handle));
    }
}

fn tag_out_of_navmesh(
    mut commands: Commands,
    agent_state: Query<(Entity, &AgentState, &AgentOf)>,
    parent: Query<(&LinearVelocity, &AiState)>,
) {
    for (entity, agent_state, agent_of) in agent_state.iter() {
        let Ok((velocity, ai_state)) = parent.get(agent_of.get()) else {
            error!("Agent parent has no velocity");
            continue;
        };
        if is_out_of_navmesh(*agent_state, *velocity, ai_state.clone()) {
            commands.entity(entity).insert(OutOfNavmesh::default());
        }
    }
}

fn despawn_out_of_navmesh(
    mut commands: Commands,
    mut out_of_navmesh: Query<(&AgentState, &mut OutOfNavmesh, &AgentOf)>,
    parent: Query<(&LinearVelocity, &AiState)>,
    time: Res<Time>,
) {
    for (agent_state, mut out_of_navmesh, agent_of) in out_of_navmesh.iter_mut() {
        let Ok((velocity, ai_state)) = parent.get(agent_of.get()) else {
            error!("Agent parent has no velocity");
            continue;
        };
        if !is_out_of_navmesh(*agent_state, *velocity, ai_state.clone()) {
            commands.entity(agent_of.get()).remove::<OutOfNavmesh>();
            continue;
        }
        out_of_navmesh.0.tick(time.delta());
        if out_of_navmesh.0.finished() {
            commands.entity(agent_of.get()).despawn();
            commands.entity(agent_of.get()).insert(Despawn);
        }
    }
}

fn is_out_of_navmesh(agent_state: AgentState, velocity: LinearVelocity, ai_state: AiState) -> bool {
    match agent_state {
        AgentState::AgentNotOnNavMesh | AgentState::NoPath => true,
        AgentState::Moving => velocity.length_squared() > 1.0,
        _ if matches!(ai_state, AiState::Chase) => velocity.length_squared() > 1.0,
        _ => false,
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct OutOfNavmesh(Timer);

impl Default for OutOfNavmesh {
    fn default() -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}
