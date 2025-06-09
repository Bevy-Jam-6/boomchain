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
    let gamee_won_sound = assets.load("audio/sound_effects/guitar-jingle-hard-rock-style.ogg");
    app.insert_resource(MenuAssets {
        background_texture,
        gamee_won_sound,
    });
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub(crate) struct MenuAssets {
    pub(crate) background_texture: Handle<Image>,
    pub(crate) gamee_won_sound: Handle<AudioSource>,
}
