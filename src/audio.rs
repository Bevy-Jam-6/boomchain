use bevy::{
    audio::{SpatialScale, Volume},
    prelude::*,
};

use crate::{
    gameplay::{
        health::{Health, OnDamage, OnDeath},
        player::Player,
        waves::{WaveFinishedPreparing, WaveStartedPreparing},
    },
    menus::Menu,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Music>();
    app.register_type::<SoundEffect>();
    app.register_type::<SpedUpSoundEffect>();
    app.add_systems(OnExit(Menu::None), suppress_soundtrack);
    app.add_systems(OnEnter(Menu::None), normalize_soundtrack);

    app.add_systems(
        Update,
        sped_up_sound_effect_setup.run_if(run_once.or(resource_changed::<GlobalVolume>)),
    );

    app.add_observer(adjust_music_to_health);
    app.add_observer(on_death);

    app.add_observer(on_wave_started);
    app.add_observer(on_wave_ended);

    app.add_systems(
        Update,
        apply_global_volume.run_if(resource_changed::<GlobalVolume>),
    );
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "music" category (e.g. global background music, soundtrack).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct Music;

/// A music audio instance.
pub(crate) fn music(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::LOOP, Music)
}

/// An organizational marker component that should be added to a spawned [`AudioPlayer`] if it's in the
/// general "sound effect" category (e.g. footsteps, the sound of a magic spell, a door opening).
///
/// This can then be used to query for and operate on sounds in that category.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct SoundEffect;

/// A sound effect audio instance.
pub(crate) fn sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (AudioPlayer(handle), PlaybackSettings::DESPAWN, SoundEffect)
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub(crate) struct SpedUpSoundEffect;

pub(crate) fn sped_up_sound_effect(handle: Handle<AudioSource>) -> impl Bundle {
    (
        AudioPlayer(handle),
        PlaybackSettings::DESPAWN,
        SpedUpSoundEffect,
    )
}

fn sped_up_sound_effect_setup(
    global_volume: Res<GlobalVolume>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink), With<SpedUpSoundEffect>>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_speed(2.0);
        sink.set_volume(playback.volume * global_volume.volume * Volume::Linear(2.0));
    }
}

/// [`GlobalVolume`] doesn't apply to already-running audio entities, so this system will update them.
fn apply_global_volume(
    global_volume: Res<GlobalVolume>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink)>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_volume(global_volume.volume * playback.volume);
    }
}

pub(crate) const DEFAULT_VOLUME: Volume = Volume::Linear(0.3);

pub(crate) fn max_volume() -> Volume {
    DEFAULT_VOLUME + Volume::Decibels(5.0)
}

fn suppress_soundtrack(
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink), With<Music>>,
    global_volume: Res<GlobalVolume>,
) {
    for (playback, mut sink) in &mut audio_query {
        let base_volume = Volume::Linear(0.75) * playback.volume * global_volume.volume;
        sink.set_volume(base_volume);

        // There is no built in pitch shifting, so halving the speed at least maintains the key of the music
        let speed_variation = 0.5;
        sink.set_speed(speed_variation);
    }
}

fn on_wave_ended(
    _trigger: Trigger<WaveStartedPreparing>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink), With<Music>>,
    global_volume: Res<GlobalVolume>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_speed(0.9);
        sink.set_volume(global_volume.volume * playback.volume * Volume::Linear(0.9));
    }
}

fn on_wave_started(
    _trigger: Trigger<WaveFinishedPreparing>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink), With<Music>>,
    global_volume: Res<GlobalVolume>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_speed(1.0);
        sink.set_volume(global_volume.volume * playback.volume / Volume::Linear(0.9));
    }
}

fn on_death(
    trigger: Trigger<OnDeath>,
    player: Query<(), With<Player>>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink), With<Music>>,
) {
    if !player.contains(trigger.target()) {
        return;
    }

    for (_, sink) in &mut audio_query {
        let speed_variation = 0.5;
        sink.set_speed(speed_variation);
    }
}

fn normalize_soundtrack(
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink), With<Music>>,
    health: Query<&Health, With<Player>>,
    global_volume: Res<GlobalVolume>,
) {
    for (playback, mut sink) in &mut audio_query {
        sink.set_volume(global_volume.volume * playback.volume);
        let speed = if health.into_iter().any(|p| p.current / p.max < 0.5) {
            1.15
        } else {
            1.0
        };
        sink.set_speed(speed);
    }
}

fn adjust_music_to_health(
    trigger: Trigger<OnDamage>,
    health: Query<&Health, With<Player>>,
    mut audio_query: Query<(&PlaybackSettings, &mut AudioSink), With<Music>>,
) {
    let entity = trigger.target();
    let Ok(health) = health.get(entity) else {
        return;
    };
    if health.current / health.max < 0.5 {
        for (_, sink) in &mut audio_query {
            sink.set_speed(1.15);
        }
    }
}
