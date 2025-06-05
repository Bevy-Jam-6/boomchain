//! NPC handling. In the demo, the NPC is a fox that moves towards the player. We can interact with the NPC to trigger dialogue.

use ai_state::AiState;
use animation::{NpcAnimationState, setup_npc_animations};
use avian3d::prelude::*;
use bevy::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use bevy_tnua::{TnuaAnimatingState, prelude::*};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use bevy_trenchbroom::prelude::*;

use crate::{
    gameplay::{
        explosion::{ExplodeOnDeath, Explosive},
        npc::stats::NpcStats,
    },
    third_party::{avian3d::CollisionLayer, bevy_trenchbroom::LoadTrenchbroomModel as _},
};

use super::{animation::AnimationPlayerAncestor, health::Health};
mod ai_state;
mod animation;
mod assets;
mod attack;
pub(crate) mod lifecycle;
pub(crate) mod navigation;
mod sound;
pub(crate) mod stats;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        navigation::plugin,
        animation::plugin,
        assets::plugin,
        sound::plugin,
        ai_state::plugin,
        attack::plugin,
        lifecycle::plugin,
        stats::plugin,
    ));
    app.register_type::<Npc>();
    app.add_observer(on_add);
}

#[derive(PointClass, Component, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[reflect(QuakeClass, Component)]
#[base(Transform, Visibility)]
#[model("models/zombie_3/zombie_3.gltf")]
#[require(NpcStats)]
// In Wasm, TrenchBroom classes are not automatically registered.
// So, we need to manually register the class in `src/third_party/bevy_trenchbroom/mod.rs`.
pub(crate) struct Npc;

pub(crate) const NPC_RADIUS: f32 = 0.4;
pub(crate) const NPC_CAPSULE_LENGTH: f32 = 0.6;
pub(crate) const NPC_HEIGHT: f32 = NPC_CAPSULE_LENGTH + 2.0 * NPC_RADIUS;

#[cfg_attr(feature = "hot_patch", hot)]
fn on_add(
    trigger: Trigger<OnAdd, NpcStats>,
    stats: Query<&NpcStats>,
    mut commands: Commands,
    assets: Res<AssetServer>,
) {
    let Ok(stats) = stats.get(trigger.target()) else {
        return;
    };
    let radius = stats.radius();
    let capsule_length = stats.capsule_length();
    let npc_float_height = stats.float_height();
    commands
        .entity(trigger.target())
        .insert((
            Npc,
            Collider::capsule(radius, capsule_length),
            TnuaController::default(),
            TnuaAvian3dSensorShape(Collider::cylinder(radius - 0.01, 0.0)),
            ColliderDensity(2_000.0),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED.unlock_rotation_y(),
            TnuaAnimatingState::<NpcAnimationState>::default(),
            AnimationPlayerAncestor,
            CollisionLayers::new(
                [CollisionLayer::Character, CollisionLayer::Npc],
                LayerMask::ALL,
            ),
            Health::new(100.0),
            AiState::default(),
            ExplodeOnDeath,
            Explosive {
                radius: stats.size * 2.5,
                impulse_strength: 5.0,
                // Scale the damage based on the NPC size
                // so that killing a larger NPC is more impactful.
                damage: stats.size * 75.0,
            },
        ))
        .with_child((
            Name::new("Npc Model"),
            SceneRoot(assets.load_trenchbroom_model::<Npc>()),
            Transform::from_xyz(0.0, -npc_float_height, 0.0).with_scale(Vec3::splat(stats.size)),
        ))
        .observe(setup_npc_animations);
}
