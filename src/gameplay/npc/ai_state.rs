use bevy::prelude::*;
use bevy_landmass::AgentState;

use super::{attack::Attacking, navigation::Agent};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<AiState>();
    app.add_systems(PreUpdate, update_ai_state);
}

#[derive(Component, Debug, Default, Reflect, Clone)]
#[reflect(Component)]
pub(crate) enum AiState {
    #[default]
    Chase,
    Attack,
}

fn update_ai_state(
    mut ai_state: Query<(Entity, &mut AiState, &Agent)>,
    agent_state: Query<&AgentState>,
    mut commands: Commands,
) {
    for (entity, mut ai_state, agent) in &mut ai_state {
        let Ok(agent_state) = agent_state.get(**agent) else {
            continue;
        };
        match ai_state.clone() {
            AiState::Chase => {
                if matches!(agent_state, AgentState::ReachedTarget) {
                    *ai_state = AiState::Attack;
                    commands.entity(entity).insert(Attacking);
                }
            }
            AiState::Attack => {
                if matches!(agent_state, AgentState::ReachedTarget) {
                    *ai_state = AiState::Chase;
                }
            }
        }
    }
}
