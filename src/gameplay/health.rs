use bevy::{prelude::*, ui::Val::*};

use crate::{PostPhysicsAppSystems, screens::Screen, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_health_hud);
    app.register_type::<Health>();
    app.add_systems(
        Update,
        kill_out_of_bounds.in_set(PostPhysicsAppSystems::TriggerDeath),
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

fn on_damage(trigger: Trigger<OnDamage>, mut health: Query<&mut Health>, mut commands: Commands) {
    let entity = trigger.target();
    let Ok(mut health) = health.get_mut(entity) else {
        return;
    };
    health.damage(trigger.event().0);
    if health.is_dead() {
        commands.entity(entity).trigger(OnDeath);
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
