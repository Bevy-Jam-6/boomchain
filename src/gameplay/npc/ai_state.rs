use bevy::prelude::*;
use bevy_landmass::AgentState;
use rand::Rng as _;

use crate::gameplay::{npc::stats::NpcStats, player::Player};

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
    mut ai_state: Query<(
        Entity,
        &mut AiState,
        &NpcStats,
        &Agent,
        &Transform,
        Has<Attacking>,
    )>,
    player: Single<&Transform, With<Player>>,
    agent_state: Query<&AgentState>,
    mut commands: Commands,
) {
    for (entity, mut ai_state, stats, agent, transform, attacking) in &mut ai_state {
        let Ok(agent_state) = agent_state.get(**agent) else {
            continue;
        };
        match ai_state.clone() {
            AiState::Chase => {
                if matches!(agent_state, AgentState::ReachedTarget) {
                    *ai_state = AiState::Attack;
                    let target = Vec3::new(
                        player.translation.x,
                        transform.translation.y,
                        player.translation.z,
                    );
                    commands.entity(entity).insert(Attacking {
                        dir: Dir3::try_from(target - transform.translation).ok(),
                        speed: rand::thread_rng().gen_range(stats.attack_speed_range.clone()),
                        damage: stats.attack_damage,
                    });
                }
            }
            AiState::Attack => {
                if !attacking {
                    *ai_state = AiState::Chase;
                }
            }
        }
    }
}
