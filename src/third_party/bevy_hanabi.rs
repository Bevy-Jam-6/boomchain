//! [Hanabi](https://github.com/djeedai/bevy_hanabi) is our GPU particle system.
//! It is used for native builds and WebGPU, but it does not work on WebGL, so we use
//! `bevy_firework` as a fallback.

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(HanabiPlugin);
}
