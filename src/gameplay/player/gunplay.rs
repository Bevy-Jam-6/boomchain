use std::time::Duration;

use bevy::prelude::*;
use bevy_enhanced_input::events::Started;

use crate::gameplay::{animation::AnimationPlayers, crosshair::CrosshairState};

use super::{animation::PlayerAnimations, default_input::Shoot};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shooting);
}

fn shooting(
    _trigger: Trigger<Started<Shoot>>,
    mut commands: Commands,
    mut query: Query<&AnimationPlayers>,
    mut q_animation: Query<(
        &PlayerAnimations,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    crosshair_state: Single<&CrosshairState>,
) {
    for anim_players in &mut query {
        let mut iter = q_animation.iter_many_mut(anim_players.iter());
        while let Some((animations, mut anim_player, mut transitions)) = iter.fetch_next() {
            if crosshair_state.wants_invisible.is_empty() {
                transitions.play(
                    &mut anim_player,
                    animations.shooting,
                    Duration::from_millis(150),
                );
            }
        }
    }
}
