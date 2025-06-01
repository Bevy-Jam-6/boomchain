use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Attacking>();
    app.add_observer(start_attack);
}

fn start_attack(_trigger: Trigger<OnAdd, Attacking>) {}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Attacking {
    pub(crate) dir: Option<Dir3>,
}
