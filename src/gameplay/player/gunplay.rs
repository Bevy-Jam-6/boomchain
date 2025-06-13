use std::time::Duration;

use super::{Player, assets::PlayerAssets, camera::PlayerCamera, default_input::Shoot};
use crate::{
    RenderLayer,
    audio::{sound_effect, sped_up_sound_effect},
    despawn_after::DespawnAfter,
    gameplay::{
        crosshair::CrosshairState,
        health::OnDamage,
        npc::Npc,
        player::{GroundCast, camera::CustomRenderLayer, camera_shake::OnTrauma},
    },
    screens::Screen,
    third_party::avian3d::CollisionLayer,
};
use avian3d::prelude::*;
use bevy::{prelude::*, render::view::RenderLayers};
use bevy_enhanced_input::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Shooting;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Reloading;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub(crate) struct WeaponStats {
    pub(crate) damage: f32,
    pub(crate) pellets: u32,
    pub(crate) spread_radius: f32,
    pub(crate) pushback: f32,
    pub(crate) extra_enemy_explosion_radius: f32,
}

pub(super) fn plugin(app: &mut App) {
    // Only until there is a weapon pickup/selection system
    app.add_observer(setup_weapon_stats);

    app.add_observer(shooting);
    app.add_observer(shooting_sounds);
    app.add_observer(shooting_sounds_reload);
    app.add_observer(spawn_muzzle_flash);
    app.add_observer(shot_pushback);

    // Only until the animations work again.
    app.add_systems(Update, remove_shooting);
    app.add_systems(Update, trigger_reload_sound);
}

fn setup_weapon_stats(trigger: Trigger<OnAdd, Player>, mut commands: Commands) {
    commands.entity(trigger.target()).insert(WeaponStats {
        damage: 5.0,
        pellets: 16,
        spread_radius: 0.15,
        pushback: 12.0,
        extra_enemy_explosion_radius: 0.0,
    });
}

fn shooting(
    trigger: Trigger<Fired<Shoot>>,
    mut commands: Commands,
    shooting: Query<(), With<Shooting>>,
    crosshair_state: Single<&CrosshairState>,
) {
    let entity = trigger.target();

    if shooting.contains(entity) || !crosshair_state.wants_invisible.is_empty() {
        return;
    }

    commands.entity(entity).insert(Shooting);
    commands.trigger(OnTrauma(0.4));
}

fn remove_shooting(
    shooting: Single<Entity, (With<Shooting>, With<Reloading>)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    mut commands: Commands,
) {
    let reload_time = 375;
    let timer = timer.get_or_insert_with(|| {
        Timer::new(Duration::from_millis(reload_time), TimerMode::Repeating)
    });
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }

    commands.entity(*shooting).remove::<Shooting>();
    commands.entity(*shooting).remove::<Reloading>();
}

fn trigger_reload_sound(
    shooting: Single<Entity, With<Shooting>>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    mut commands: Commands,
) {
    // The name is not precise, we simply start the reload time after this time of the shooting sound (they overlap a little)
    let shooting_sound_len = 175;
    let timer = timer.get_or_insert_with(|| {
        Timer::new(
            Duration::from_millis(shooting_sound_len),
            TimerMode::Repeating,
        )
    });
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }

    commands.entity(*shooting).insert(Reloading);
}

fn shooting_sounds(
    _trigger: Trigger<OnAdd, Shooting>,
    mut commands: Commands,
    mut player_assets: ResMut<PlayerAssets>,
    state: Res<State<Screen>>,
) {
    if *state != Screen::Gameplay {
        return;
    }

    let rng = &mut rand::thread_rng();
    let shooting_sound = player_assets.shooting_sounds.pick(rng).clone();

    commands.spawn(sound_effect(shooting_sound));
}

fn shooting_sounds_reload(
    _trigger: Trigger<OnAdd, Reloading>,
    mut commands: Commands,
    player_assets: ResMut<PlayerAssets>,
    state: Res<State<Screen>>,
) {
    if *state != Screen::Gameplay {
        return;
    }

    commands.spawn(sound_effect(player_assets.reload_sound.clone()));
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_muzzle_flash(
    _trigger: Trigger<OnAdd, Shooting>,
    cam: Single<Entity, With<PlayerCamera>>,
    mut commands: Commands,
) {
    commands.entity(*cam).with_children(|parent| {
        parent.spawn((
            Transform::from_xyz(-0.45, -0.1, -3.8),
            DespawnAfter::new(Duration::from_millis(200)),
            PointLight {
                intensity: 7000.0,
                shadows_enabled: false,
                ..default()
            },
            RenderLayers::from(RenderLayer::VIEW_MODEL),
            CustomRenderLayer,
        ));
        parent.spawn((
            Transform::from_xyz(-0.5, -0.1, -0.0),
            DespawnAfter::new(Duration::from_millis(200)),
            PointLight {
                intensity: 23500.0,
                shadows_enabled: true,
                #[cfg(feature = "native")]
                soft_shadows_enabled: true,
                ..default()
            },
            RenderLayers::from(RenderLayer::DEFAULT),
            CustomRenderLayer,
        ));
    });
}

fn shot_pushback(
    trigger: Trigger<OnAdd, Shooting>,
    mut player: Query<(&mut LinearVelocity, &GroundCast, &WeaponStats), With<Player>>,
    player_camera_parent: Single<&Transform, With<PlayerCamera>>,
) {
    let Ok((mut lin_vel, ground_cast, weapon_stats)) = player.get_mut(trigger.target()) else {
        return;
    };
    let back = player_camera_parent.back();

    if ground_cast.is_none() {
        // Apply pushback
        lin_vel.0 += back * weapon_stats.pushback;
    }
}
