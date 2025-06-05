//! Preload NPC assets.

use bevy::prelude::*;
use bevy_shuffle_bag::ShuffleBag;

use crate::{
    asset_tracking::LoadResource, third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _,
};

use super::Npc;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<NpcAssets>();
    app.load_resource::<NpcAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub(crate) struct NpcAssets {
    #[dependency]
    pub(crate) _model: Handle<Scene>,
    #[dependency]
    pub(crate) gib_head: Handle<Scene>,
    #[dependency]
    pub(crate) gib_torso: Handle<Scene>,
    #[dependency]
    pub(crate) gib_arm_1: Handle<Scene>,
    #[dependency]
    pub(crate) gib_arm_2: Handle<Scene>,
    #[dependency]
    pub(crate) gib_leg: Handle<Scene>,
    #[dependency]
    pub(crate) gib_foot: Handle<Scene>,
    #[dependency]
    pub(crate) gib_pelvis: Handle<Scene>,
    #[dependency]
    pub(crate) idle_animation: Handle<AnimationClip>,
    #[dependency]
    pub(crate) walk_animation: Handle<AnimationClip>,
    #[dependency]
    pub(crate) attack_animation: Handle<AnimationClip>,
    #[dependency]
    pub(crate) steps: ShuffleBag<Handle<AudioSource>>,
    #[dependency]
    pub(crate) death_sound: ShuffleBag<Handle<AudioSource>>,
    #[dependency]
    pub(crate) idle_sound: ShuffleBag<Handle<AudioSource>>,
    #[dependency]
    pub(crate) stagger_sound: ShuffleBag<Handle<AudioSource>>,
    #[dependency]
    pub(crate) attack_sound: ShuffleBag<Handle<AudioSource>>,
}

impl FromWorld for NpcAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let rng = &mut rand::thread_rng();
        Self {
            gib_head: assets.load("models/zombie_3/gib_head.gltf#Scene0"),
            gib_torso: assets.load("models/zombie_3/gib_torso.gltf#Scene0"),
            gib_arm_1: assets.load("models/zombie_3/gib_arm_1.gltf#Scene0"),
            gib_arm_2: assets.load("models/zombie_3/gib_arm_2.gltf#Scene0"),
            gib_leg: assets.load("models/zombie_3/gib_leg.gltf#Scene0"),
            gib_foot: assets.load("models/zombie_3/gib_foot.gltf#Scene0"),
            gib_pelvis: assets.load("models/zombie_3/gib_pelvis.gltf#Scene0"),
            _model: assets.load(Npc::scene_path()),
            attack_animation: assets.load(Npc::animation_path(0)),
            idle_animation: assets.load(Npc::animation_path(1)),
            walk_animation: assets.load(Npc::animation_path(2)),
            steps: ShuffleBag::try_new(
                [
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_01.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_02.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_03.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_04.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_05.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_06.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_07.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_08.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_09.ogg"),
                    assets.load("audio/sound_effects/run/Footsteps_Rock_Run_10.ogg"),
                ],
                rng,
            )
            .unwrap(),
            death_sound: ShuffleBag::try_new(
                [
                    assets.load("audio/sound_effects/impact/Impact02.ogg"),
                    assets.load("audio/sound_effects/impact/Impact04.ogg"),
                    assets.load("audio/sound_effects/impact/Impact08.ogg"),
                ],
                rng,
            )
            .unwrap(),
            idle_sound: ShuffleBag::try_new(
                [
                    assets.load("audio/sound_effects/zombie/01-intimidating-breathing.wav"),
                    assets.load("audio/sound_effects/zombie/02-pained-exhale.wav"),
                    assets.load("audio/sound_effects/zombie/04-shriek.wav"),
                    assets.load("audio/sound_effects/zombie/05-idle-breathing.wav"),
                    assets.load("audio/sound_effects/zombie/08-roar.wav"),
                    //assets.load("audio/sound_effects/zombie/09-tantrum.wav"),
                    assets.load("audio/sound_effects/zombie/10-conversational.wav"),
                    assets.load("audio/sound_effects/zombie/11-weak-defiance.wav"),
                ],
                rng,
            )
            .unwrap(),
            stagger_sound: ShuffleBag::try_new(
                [
                    assets.load("audio/sound_effects/zombie/03-lunging-snarl.wav"),
                    assets.load("audio/sound_effects/zombie/07-grunt.wav"),
                    //assets.load("audio/sound_effects/zombie/06-growl.wav"),
                ],
                rng,
            )
            .unwrap(),
            attack_sound: ShuffleBag::try_new([assets.load("audio/sound_effects/throw.ogg")], rng)
                .unwrap(),
        }
    }
}
