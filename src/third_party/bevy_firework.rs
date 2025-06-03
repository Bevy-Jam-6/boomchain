//! [`bevy_firework`](https://github.com/mbrea-c/bevy_firework) is our CPU particle system.
//! Unlike Hanabi, it supports both WebGPU and WebGL.

use bevy::prelude::*;
use bevy_firework::plugin::ParticleSystemPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(ParticleSystemPlugin::default());
}
