use bevy::prelude::*;

pub(crate) fn plugin(app: &mut App) {
    app.register_type::<FontAssets>();

    // We need to do this here instead of at `Startup` because of some ordering shenanigans
    let assets = app.world().resource::<AssetServer>();
    let font_handle: Handle<Font> = assets.load("fonts/Jersey25-Regular.ttf");
    app.insert_resource(FontAssets {
        font: font_handle.clone(),
    });

    app.add_systems(Update, setup_default_font);
}

#[derive(Resource, Clone, Debug, Default, Reflect)]
struct FontAssets {
    font: Handle<Font>,
}

fn setup_default_font(
    mut fonts: ResMut<Assets<Font>>,
    font_assets: Res<FontAssets>,
    mut ran: Local<bool>,
) {
    // This is ugly, but I couldn't get it working with asset events or the `load_resource` approach :/

    if *ran {
        return;
    }

    let Some(font) = fonts.get(&font_assets.font).cloned() else {
        return;
    };

    // Set the default font to the loaded font
    *fonts.get_mut(Handle::default().id()).unwrap() = font.clone();
    *ran = true;
}
