use std::time::Duration;

use bevy::{
    audio::{SpatialScale, Volume},
    color::palettes::css::ORANGE,
    prelude::*,
    render::view::RenderLayers,
};
use bevy_firework::{
    bevy_utilitarian::prelude::RandF32,
    core::{BlendMode, ParticleSpawner, SpawnTransformMode},
    curve::FireworkGradient,
    emission_shape::EmissionShape,
};

use bevy_mesh_decal::spray_decal;
use rand::Rng as _;

use super::OnExplode;
use crate::{
    RenderLayer,
    audio::SoundEffect,
    despawn_after::DespawnAfter,
    gameplay::{
        explosion::ExplodeOnDeath,
        gore_settings::{Gore, GoreSettings},
        health::OnDeath,
        npc::stats::NpcStats,
        player::{Player, gunplay::WeaponStats},
    },
    platform_support::is_webgpu_or_native,
    screens::Screen,
};

const EXPLOSION_LIGHT_INTENSITY: f32 = 50_000.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PropExplosionVfx>();
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Debug, Default)]
pub(crate) struct PropExplosionVfx;
