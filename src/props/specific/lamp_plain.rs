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
    app.register_type::<LampPlain>();
}

#[derive(PointClass, Component, Debug, Reflect)]
#[reflect(QuakeClass, Component)]
#[base(Transform, Visibility)]
#[model("models/darkmod/lights/non-extinguishable/electric_plain1_unattached.gltf")]
#[spawn_hooks(SpawnHooks::new().preload_model::<Self>())]
#[classname("light_lamp_plain")]
struct LampPlain {
    color: Color,
    intensity: f32,
}

impl Default for LampPlain {
    fn default() -> Self {
        Self {
            color: Color::srgb_u8(180, 210, 255),
            intensity: 25_000.0,
        }
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn setup_lamp_wall_electric(
    trigger: Trigger<OnAdd, LampPlain>,
    lamp: Query<&LampPlain>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let lamp = lamp.get(trigger.target()).unwrap();

    let model = asset_server.load_trenchbroom_model::<LampPlain>();
    commands
        .entity(trigger.target())
        .insert(SceneRoot(model))
        .with_child((
            Transform::from_xyz(0.0, -0.08, 0.0),
            PointLight {
                color: lamp.color,
                intensity: lamp.intensity,
                range: 8.0,
                shadows_enabled: false,
                ..default()
            },
        ))
        .observe(disable_shadow_casting_on_instance_ready);
}
