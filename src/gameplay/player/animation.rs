//! Player animations.

use std::time::Duration;

use bevy::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, prelude::*};

use crate::{
    PostPhysicsAppSystems,
    gameplay::{animation::AnimationPlayers, crosshair::CrosshairState, player::gunplay::Shooting},
};

use super::assets::PlayerAssets;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PlayerAnimations>();
    app.add_systems(
        Update,
        play_animations.in_set(PostPhysicsAppSystems::PlayAnimations),
    );
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct PlayerAnimations {
    hidden: AnimationNodeIndex,
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    shoot: AnimationNodeIndex,
}

#[cfg_attr(feature = "hot_patch", hot)]
pub(crate) fn setup_player_animations(
    trigger: Trigger<OnAdd, AnimationPlayers>,
    q_anim_players: Query<&AnimationPlayers>,
    mut commands: Commands,
    assets: Res<PlayerAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let anim_players = q_anim_players.get(trigger.target()).unwrap();
    for anim_player in anim_players.iter() {
        let (graph, indices) = AnimationGraph::from_clips([
            assets.hidden_animation.clone(),
            assets.idle_animation.clone(),
            assets.shoot_animation.clone(),
            assets.walk_animation.clone(),
        ]);
        let [hidden_index, idle_index, shoot_index, walk_index] = indices.as_slice() else {
            unreachable!()
        };
        let graph_handle = graphs.add(graph);

        let animations = PlayerAnimations {
            hidden: *hidden_index,
            idle: *idle_index,
            shoot: *shoot_index,
            walk: *walk_index,
        };
        let transitions = AnimationTransitions::new();
        commands.entity(anim_player).insert((
            animations,
            AnimationGraphHandle(graph_handle),
            transitions,
        ));
    }
}

/// Managed by [`play_animations`]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) enum PlayerAnimationState {
    #[default]
    Hidden,
    Idle,
    Walking,
    Shooting,
}

#[cfg_attr(feature = "hot_patch", hot)]
fn play_animations(
    mut query: Query<(
        Entity,
        &mut TnuaAnimatingState<PlayerAnimationState>,
        &TnuaController,
        &AnimationPlayers,
        Has<Shooting>,
    )>,
    mut q_animation: Query<(
        &PlayerAnimations,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    crosshair_state: Single<&CrosshairState>,
    mut commands: Commands,
) {
    for (entity, mut animating_state, controller, anim_players, is_shooting) in &mut query {
        let mut iter = q_animation.iter_many_mut(anim_players.iter());

        let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
            continue;
        };
        let speed = basis_state.running_velocity.length();
        const WALK_SPEED: f32 = 0.2;
        while let Some((animations, mut anim_player, mut transitions)) = iter.fetch_next() {
            match animating_state.update_by_discriminant(
                // we show the player's hands exactly if and only if the crosshair is visible
                if crosshair_state.wants_invisible.is_empty() {
                    if is_shooting {
                        if anim_player.is_playing_animation(animations.shoot)
                            && anim_player.all_finished()
                        {
                            commands.entity(entity).remove::<Shooting>();
                            if speed > WALK_SPEED {
                                PlayerAnimationState::Walking
                            } else {
                                PlayerAnimationState::Idle
                            }
                        } else {
                            PlayerAnimationState::Shooting
                        }
                    } else if speed > WALK_SPEED {
                        PlayerAnimationState::Walking
                    } else {
                        PlayerAnimationState::Idle
                    }
                } else {
                    PlayerAnimationState::Hidden
                },
            ) {
                TnuaAnimatingStateDirective::Maintain { .. } => {}
                TnuaAnimatingStateDirective::Alter {
                    // We don't need the old state here, but it's available for transition
                    // animations.
                    old_state: _,
                    state,
                } => match state {
                    PlayerAnimationState::Hidden => {
                        transitions.play(
                            &mut anim_player,
                            animations.hidden,
                            Duration::from_millis(400),
                        );
                    }
                    PlayerAnimationState::Idle => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.idle,
                                Duration::from_millis(300),
                            )
                            .repeat();
                    }
                    PlayerAnimationState::Shooting => {
                        transitions.play(
                            &mut anim_player,
                            animations.shoot,
                            Duration::from_millis(50),
                        );
                    }
                    PlayerAnimationState::Walking => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.walk,
                                Duration::from_millis(300),
                            )
                            .repeat();
                    }
                },
            }
        }
    }
}
