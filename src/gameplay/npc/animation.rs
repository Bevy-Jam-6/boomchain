//! NPC animation handling.

use std::time::Duration;

use bevy::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use bevy_tnua::{TnuaAnimatingState, TnuaAnimatingStateDirective, prelude::*};
use rand::Rng;

use crate::{PostPhysicsAppSystems, gameplay::animation::AnimationPlayers};

use super::{assets::NpcAssets, attack::Attacking};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<NpcAnimations>();
    app.add_systems(
        Update,
        play_animations.in_set(PostPhysicsAppSystems::PlayAnimations),
    );
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct NpcAnimations {
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
    attack: AnimationNodeIndex,
}

#[cfg_attr(feature = "hot_patch", hot)]
pub(crate) fn setup_npc_animations(
    trigger: Trigger<OnAdd, AnimationPlayers>,
    q_anim_players: Query<&AnimationPlayers>,
    mut commands: Commands,
    assets: Res<NpcAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let anim_players = q_anim_players.get(trigger.target()).unwrap();
    for anim_player in anim_players.iter() {
        let (graph, indices) = AnimationGraph::from_clips([
            assets.attack_animation.clone(),
            assets.idle_animation.clone(),
            assets.walk_animation.clone(),
        ]);
        let [attack_index, idle_index, walk_index] = indices.as_slice() else {
            unreachable!()
        };
        let graph_handle = graphs.add(graph);

        let animations = NpcAnimations {
            idle: *idle_index,
            walk: *walk_index,
            attack: *attack_index,
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NpcAnimationState {
    Standing,
    Airborne,
    Attack(f32),
    Walking(f32),
}

#[cfg_attr(feature = "hot_patch", hot)]
fn play_animations(
    mut query: Query<(
        Entity,
        &mut TnuaAnimatingState<NpcAnimationState>,
        &TnuaController,
        &AnimationPlayers,
        Option<&Attacking>,
    )>,
    mut q_animation: Query<(
        &NpcAnimations,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    mut commands: Commands,
) {
    for (entity, mut animating_state, controller, anim_players, attacking) in &mut query {
        let mut iter = q_animation.iter_many_mut(anim_players.iter());
        while let Some((animations, mut anim_player, mut transitions)) = iter.fetch_next() {
            match animating_state.update_by_discriminant({
                if let Some(attacking) = attacking {
                    if anim_player.is_playing_animation(animations.attack)
                        && anim_player.all_finished()
                    {
                        commands.entity(entity).remove::<Attacking>();
                        NpcAnimationState::Standing
                    } else {
                        NpcAnimationState::Attack(attacking.speed)
                    }
                } else {
                    let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>()
                    else {
                        continue;
                    };
                    let speed = basis_state.running_velocity.length();
                    if controller.is_airborne().unwrap() {
                        NpcAnimationState::Airborne
                    } else if speed > 0.01 {
                        NpcAnimationState::Walking(speed)
                    } else {
                        NpcAnimationState::Standing
                    }
                }
            }) {
                TnuaAnimatingStateDirective::Maintain { state } => {
                    if let NpcAnimationState::Walking(speed) = state
                        && let Some((_index, playing_animation)) =
                            anim_player.playing_animations_mut().next()
                    {
                        let anim_speed = (speed / 5.0).max(0.2);
                        playing_animation.set_speed(anim_speed);
                    }
                }
                TnuaAnimatingStateDirective::Alter {
                    // We don't need the old state here, but it's available for transition
                    // animations.
                    old_state: _,
                    state,
                } => match state {
                    NpcAnimationState::Airborne => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.walk,
                                Duration::from_millis(200),
                            )
                            .repeat();
                    }
                    NpcAnimationState::Standing => {
                        transitions
                            .play(
                                &mut anim_player,
                                animations.idle,
                                Duration::from_millis(500),
                            )
                            .repeat();
                    }
                    NpcAnimationState::Attack(speed) => {
                        let transition_millis = rand::thread_rng().gen_range(100..=300);
                        transitions
                            .play(
                                &mut anim_player,
                                animations.attack,
                                Duration::from_millis(transition_millis),
                            )
                            .set_speed(*speed);
                    }
                    NpcAnimationState::Walking(_speed) => {
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
