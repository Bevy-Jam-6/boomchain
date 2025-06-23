use bevy::prelude::*;

use crate::{PostPhysicsAppSystems, screens::Screen};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Health>();
    app.add_systems(
        Update,
        kill_out_of_bounds
            .in_set(PostPhysicsAppSystems::TriggerDeath)
            .run_if(in_state(Screen::Gameplay)),
    );
    app.add_observer(on_damage);
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Health {
    pub(crate) current: f32,
    pub(crate) max: f32,
}

impl Health {
    pub(crate) fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub(crate) fn fraction(&self) -> f32 {
        self.current / self.max
    }

    pub(crate) fn heal_full(&mut self) {
        self.current = self.max;
    }

    fn damage(&mut self, amount: f32) {
        self.current -= amount;
        self.current = self.current.max(0.0);
    }

    pub(crate) fn is_dead(&self) -> bool {
        self.current <= 0.0
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: 100.0,
            max: 100.0,
        }
    }
}

fn on_damage(trigger: Trigger<OnDamage>, mut health: Query<&mut Health>, mut commands: Commands) {
    let entity = trigger.target();
    let Ok(mut health) = health.get_mut(entity) else {
        return;
    };
    health.damage(trigger.event().0);
    if health.is_dead() {
        commands.entity(entity).remove::<Health>().trigger(OnDeath);
    }
}

#[derive(Debug, Event)]
pub(crate) struct OnDamage(pub(crate) f32);

#[derive(Debug, Event)]
pub(crate) struct OnDeath;

fn kill_out_of_bounds(health: Query<(Entity, &Transform)>, mut commands: Commands) {
    for (entity, transform) in health.iter() {
        if transform.translation.y < -300.0 {
            commands.entity(entity).trigger(OnDeath);
        }
    }
}
