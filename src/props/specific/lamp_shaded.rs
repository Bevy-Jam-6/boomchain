use std::f32::consts::TAU;

use bevy::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use bevy_trenchbroom::prelude::*;

use crate::{
    props::effects::disable_shadow_casting_on_instance_ready,
    third_party::bevy_trenchbroom::LoadTrenchbroomModel as _,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_lamp_shaded);
    app.register_type::<LampShaded>();
}

#[derive(PointClass, Component, Debug, Reflect)]
#[reflect(QuakeClass, Component)]
#[base(Transform, Visibility)]
#[model("models/darkmod/lights/non-extinguishable/lamp_shaded03/lamp_shaded03.gltf")]
#[spawn_hooks(SpawnHooks::new().preload_model::<Self>())]
#[classname("light_lamp_shaded03")]
pub(crate) struct LampShaded;

#[cfg_attr(feature = "hot_patch", hot)]
fn setup_lamp_shaded(
    trigger: Trigger<OnAdd, LampShaded>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let model = asset_server.load_trenchbroom_model::<LampShaded>();
    commands
        .entity(trigger.target())
        .insert((
            SceneRoot(model),
            children![(
                SpotLight {
                    color: Color::srgb_u8(232, 199, 176),
                    intensity: 800_000.0,
                    radius: 0.1,
                    shadows_enabled: true,
                    #[cfg(feature = "native")]
                    soft_shadows_enabled: true,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.1, -0.25)
                    .with_rotation(Quat::from_rotation_x(TAU / 4.5)),
            )],
        ))
        .observe(disable_shadow_casting_on_instance_ready);
}
