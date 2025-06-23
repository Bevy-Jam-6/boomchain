use bevy::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use bevy_trenchbroom::prelude::*;

use crate::{
    props::effects::disable_shadow_casting_on_instance_ready,
    third_party::bevy_trenchbroom::LoadTrenchbroomModel as _,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_lamp_wall_electric);
    app.register_type::<LampWallElectric>();
}

#[derive(PointClass, Component, Debug, Reflect)]
#[reflect(QuakeClass, Component)]
#[base(Transform, Visibility)]
#[model(
    "models/darkmod/lights/non-extinguishable/lamp_wall_electric_01/lamp_wall_electric_01.gltf"
)]
#[spawn_hooks(SpawnHooks::new().preload_model::<Self>())]
#[classname("light_lamp_wall_electric")]
pub(crate) struct LampWallElectric;

#[cfg_attr(feature = "hot_patch", hot)]
fn setup_lamp_wall_electric(
    trigger: Trigger<OnAdd, LampWallElectric>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let model = asset_server.load_trenchbroom_model::<LampWallElectric>();
    commands
        .entity(trigger.target())
        .insert(SceneRoot(model))
        .with_child((
            Transform::from_xyz(0.0, -0.08, -0.35),
            PointLight {
                color: Color::srgb_u8(232, 199, 176),
                intensity: 40_000.0,
                range: 8.0,
                shadows_enabled: false,
                ..default()
            },
        ))
        .observe(disable_shadow_casting_on_instance_ready);
}
