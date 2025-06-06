use bevy::{ecs::relationship::Relationship, prelude::*};

use crate::{
    audio::sound_effect,
    gameplay::{
        health::{Health, OnDamage},
        player::{Player, assets::PlayerAssets, camera_shake::OnTrauma},
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(shake_on_hit);
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
