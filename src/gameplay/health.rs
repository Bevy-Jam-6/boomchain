use bevy::{prelude::*, ui::Val::*};

use crate::{PostPhysicsAppSystems, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_health_hud);
    app.register_type::<Health>();
    app.add_systems(
        Update,
        (kill_out_of_bounds, trigger_death)
            .chain()
            .in_set(PostPhysicsAppSystems::TriggerDeath),
    );
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

    pub(crate) fn damage(&mut self, amount: f32) {
        self.current -= amount;
        self.current = self.current.max(0.0);
    }

    pub(crate) fn kill(&mut self) {
        self.current = 0.0;
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

fn spawn_health_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Health HUD"),
        GlobalZIndex(2),
        StateScoped(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            width: Percent(100.0),
            height: Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            bottom: Px(20.0),
            ..default()
        },
        // Don't block picking events for other UI roots.
        Pickable::IGNORE,
        children![widget::label("Health")],
    ));
}

fn trigger_death(health: Query<(Entity, &Health), Changed<Health>>, mut commands: Commands) {
    for (entity, health) in health.iter() {
        if health.is_dead() {
            commands.entity(entity).trigger(OnDeath);
        }
    }
}

#[derive(Debug, Event)]
pub(crate) struct OnDeath;

fn kill_out_of_bounds(mut health: Query<(&mut Health, &Transform)>) {
    for (mut health, transform) in health.iter_mut() {
        if transform.translation.y < -300.0 {
            health.kill();
        }
    }
}
