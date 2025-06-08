use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MenuAssets>();

    let assets = app.world().resource::<AssetServer>();
    let background_texture = assets.load({
        #[cfg(feature = "dev")]
        {
            "images/blood/BloodFabric04Grayscale.png"
        }
        #[cfg(not(feature = "dev"))]
        {
            "images/blood/BloodFabric04Grayscale.ktx2"
        }
    });
    app.insert_resource(MenuAssets { background_texture });
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub(crate) struct MenuAssets {
    pub(crate) background_texture: Handle<Image>,
}
