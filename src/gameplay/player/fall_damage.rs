use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    gameplay::{
        health::{Health, OnDamage},
        player::{GroundCast, Player},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        apply_fall_damage.run_if(in_state(Screen::Gameplay)),
    );
}

const FALL_DAMAGE_THRESHOLD: f32 = 35.0;

fn apply_fall_damage(
    mut commands: Commands,
    player: Single<(Entity, &Health, &GroundCast, &LinearVelocity), With<Player>>,
    mut local: Local<(bool, f32)>,
) {
    let (was_airborne, last_y_speed) = &mut *local;
    let (entity, health, ground_cast, velocity) = *player;
    let is_airborne = ground_cast.is_none();
    if is_airborne {
        *was_airborne = true;
        *last_y_speed = velocity.y.abs();
        return;
    }
    if !*was_airborne {
        *last_y_speed = velocity.y.abs();
        return;
    }
    *was_airborne = false;

    if *last_y_speed < FALL_DAMAGE_THRESHOLD {
        *last_y_speed = velocity.y.abs();
        return;
    }

    let max_damage = health.max / 4.0;
    let damage = (1.5 * (*last_y_speed - FALL_DAMAGE_THRESHOLD)).min(max_damage);

    commands.trigger_targets(OnDamage(damage), entity);

    *last_y_speed = velocity.y.abs();
}
