use bevy::prelude::*;

use crate::gameplay::{
    health::OnDamage,
    player::{Player, camera_shake::OnTrauma},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shake_on_hit);
}

fn shake_on_hit(
    trigger: Trigger<OnDamage>,
    player: Query<(), With<Player>>,
    mut commands: Commands,
) {
    if !player.contains(trigger.target()) {
        return;
    }
    let base_trauma = 0.7 / 10.0;
    let dmg = trigger.event().0;
    commands
        .entity(trigger.target())
        .trigger(OnTrauma(base_trauma * dmg));
}
