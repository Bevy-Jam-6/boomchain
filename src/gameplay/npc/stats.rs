use std::ops::Range;

use bevy::prelude::*;

use crate::gameplay::health::Health;

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
    mut npc: Query<(&NpcStats, &mut Transform)>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let Ok((stats, transform)) = npc.get_mut(entity) else {
        return;
    };
    commands.entity(entity).insert(Health::new(stats.health));
}
