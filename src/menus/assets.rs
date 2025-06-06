use bevy::prelude::*;

use crate::asset_tracking::LoadResource;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MenuAssets>();
    app.load_resource::<MenuAssets>();
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub(crate) struct MenuAssets {
    #[dependency]
    pub(crate) background_texture_1: Handle<Image>,
    #[dependency]
    pub(crate) background_texture_2: Handle<Image>,
}

impl FromWorld for MenuAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            background_texture_1: assets.load("images/blood/BloodFabric04Grayscale.png"),
            background_texture_2: assets.load("images/blood/BloodFabric07Grayscale.png"),
        }
    }
}
