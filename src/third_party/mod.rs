//! Third-party plugins.
//!
//! We use one file per plugin to encapsulate setup or boilerplate necessary for that plugin.
//! Many plugins don't require any setup, but it's still nice to have them in an own file so
//! that we are ready to add convenience methods or similar when needed.

use bevy::prelude::*;

use crate::platform_support::is_webgpu_or_native;

pub(crate) mod avian3d;
mod bevy_enhanced_input;
mod bevy_firework;
mod bevy_framepace;
pub(crate) mod bevy_landmass;
mod bevy_mesh_decal;
mod bevy_tnua;
pub(crate) mod bevy_trenchbroom;
mod fixes;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        fixes::plugin,
        bevy_trenchbroom::plugin,
        avian3d::plugin,
        bevy_enhanced_input::plugin,
        bevy_firework::plugin,
        bevy_tnua::plugin,
        bevy_landmass::plugin,
        bevy_mesh_decal::plugin,
        bevy_framepace::plugin,
    ));
}
