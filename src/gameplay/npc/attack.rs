use bevy::prelude::*;

use crate::gameplay::player::Player;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Attacking>();
    app.add_observer(start_attack);
}

fn start_attack(trigger: Trigger<OnAdd, Attacking>) {}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct Attacking;
