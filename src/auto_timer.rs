use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<AutoTimer>();
    app.add_systems(Update, auto_tick);
}

/// A [`Timer`] component that automatically starts ticking when added to an entity,
/// and triggers the [`AutoTimerFinished`] event when it finishes.
#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[reflect(Component, Debug, Default)]
pub struct AutoTimer(pub Timer);

/// An event that is triggered when an [`AutoTimer`] finishes ticking.
#[derive(Event)]
pub struct OnAutoTimerFinish;

fn auto_tick(mut commands: Commands, time: Res<Time>, mut query: Query<(&mut AutoTimer, Entity)>) {
    for (mut timer, e) in query.iter_mut() {
        if !timer.paused() {
            timer.tick(time.delta());
            if timer.just_finished() {
                commands.trigger_targets(OnAutoTimerFinish, e);
            }
        }
    }
}
