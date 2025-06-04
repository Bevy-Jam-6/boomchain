use std::ops::Range;

use bevy::prelude::*;

use crate::gameplay::{
    health::Health,
    npc::{NPC_CAPSULE_LENGTH, NPC_RADIUS},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<NpcStats>();
    app.add_observer(apply_initial_stats);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct NpcStats {
    pub health: f32,
    pub desired_speed: f32,
    pub max_speed: f32,
    pub attack_damage: f32,
    pub attack_speed_range: Range<f32>,
    pub size: f32,
}

impl NpcStats {
    pub fn radius(&self) -> f32 {
        NPC_RADIUS * self.size
    }

    pub fn capsule_length(&self) -> f32 {
        NPC_CAPSULE_LENGTH * self.size
    }

    pub fn height(&self) -> f32 {
        self.capsule_length() + 2.0 * self.radius()
    }

    pub fn half_height(&self) -> f32 {
        self.height() / 2.0
    }

    pub fn float_height(&self) -> f32 {
        self.half_height() + 0.5
    }
}

impl Default for NpcStats {
    fn default() -> Self {
        Self {
            health: 100.0,
            desired_speed: 10.0,
            max_speed: 10.0,
            attack_damage: 10.0,
            attack_speed_range: 1.2..2.1,
            size: 1.0,
        }
    }
}

fn apply_initial_stats(
    trigger: Trigger<OnAdd, NpcStats>,
    mut npc: Query<&NpcStats>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok(stats) = npc.get_mut(entity) else {
        return;
    };
    commands.entity(entity).insert(Health::new(stats.health));
}
