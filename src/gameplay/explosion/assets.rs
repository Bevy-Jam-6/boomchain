//! Preload explosion assets.

use bevy::prelude::*;
use bevy_hanabi::EffectAsset;
use bevy_shuffle_bag::ShuffleBag;

use super::effects::{hanabi_enemy_explosion, hanabi_prop_explosion};
use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<ExplosionAssets>();
    app.load_resource::<ExplosionAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct ExplosionAssets {
    #[dependency]
    pub(crate) prop_explosion_sfx: ShuffleBag<Handle<AudioSource>>,
    pub(crate) prop_explosion_vfx: Handle<EffectAsset>,
    pub(crate) enemy_explosion_vfx: Handle<EffectAsset>,
    pub(crate) blood_splatter: ShuffleBag<Handle<StandardMaterial>>,
}

impl FromWorld for ExplosionAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        let rng = &mut rand::thread_rng();

        let prop_explosion_sfx = ShuffleBag::try_new(
            [
                assets.load("audio/sound_effects/explosion/SE-Explosion3-A.ogg"),
                assets.load("audio/sound_effects/explosion/SE-Explosion3-B.ogg"),
                assets.load("audio/sound_effects/explosion/SE-Explosion3-C.ogg"),
                assets.load("audio/sound_effects/explosion/SE-Explosion3-F.ogg"),
            ],
            rng,
        )
        .unwrap();

        let create_material = |world: &mut World, path: &str| -> Handle<StandardMaterial> {
            let texture = world.resource::<AssetServer>().load(path);
            world.add_asset(StandardMaterial {
                base_color_texture: Some(texture),
                perceptual_roughness: 0.4,
                metallic: 0.2,
                reflectance: 0.2,
                alpha_mode: AlphaMode::Mask(0.5),
                ..default()
            })
        };
        let blood_splatter = ShuffleBag::try_new(
            [
                create_material(world, "images/blood/BloodFabric01.png"),
                create_material(world, "images/blood/BloodFabric07.png"),
                create_material(world, "images/blood/BloodFabric12.png"),
            ],
            rng,
        )
        .unwrap();

        let prop_explosion_vfx = hanabi_prop_explosion(world);
        let enemy_explosion_vfx = hanabi_enemy_explosion(world);

        Self {
            prop_explosion_sfx,
            prop_explosion_vfx: world.add_asset(prop_explosion_vfx),
            enemy_explosion_vfx: world.add_asset(enemy_explosion_vfx),
            blood_splatter,
        }
    }
}
