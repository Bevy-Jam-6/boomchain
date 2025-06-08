use std::{any::Any, iter::once};

use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    Pause,
    font::FontAssets,
    gameplay::{
        crosshair::CrosshairState,
        health::Health,
        player::{
            Player,
            default_input::{BlocksInput, OpenUpgradeMenu},
            gunplay::WeaponStats,
            movement::MovementStats,
        },
        waves::{WaveFinishedPreparing, WaveStartedPreparing},
    },
    screens::Screen,
    theme::widget::{button, ui_root},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(offer_upgrades);
    app.add_observer(spawn_upgrade_ui);
    app.add_observer(unoffer_upgrades);
    app.add_observer(despawn_upgrades);
    app.add_systems(
        Update,
        (pause_in_menu, hide_upgrade_menu_on_pause).run_if(any_with_component::<UpgradeMenu>),
    );
}

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub(crate) struct Upgrades(Vec<Upgrade>);

#[derive(Reflect, Debug, Clone, Copy)]
pub(crate) enum Upgrade {
    Health,
    Damage,
    Speed,
    Accuracy,
    Bullets,
}
impl Upgrade {
    fn all_except_health() -> Vec<Upgrade> {
        // Add new upgrades here!
        vec![Upgrade::Damage, Upgrade::Speed, Upgrade::Accuracy]
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct UpgradeMenu;

fn offer_upgrades(_trigger: Trigger<WaveStartedPreparing>, mut commands: Commands) {
    let available_upgrades = Upgrade::all_except_health();
    let upgrades = available_upgrades
        .choose_multiple(&mut rand::thread_rng(), 2)
        .copied();
    // Healing is always available.
    let upgrades = once(Upgrade::Health).chain(upgrades).collect();
    commands.spawn((Upgrades(upgrades), StateScoped(Screen::Gameplay)));
}

fn spawn_upgrade_ui(
    _trigger: Trigger<Fired<OpenUpgradeMenu>>,
    upgrades: Single<&Upgrades>,
    upgrade_menus: Query<(), With<UpgradeMenu>>,
    mut commands: Commands,
    mut block_input: ResMut<BlocksInput>,
    mut crosshair_state: Single<&mut CrosshairState>,
    fonts: Res<FontAssets>,
) {
    if !upgrade_menus.is_empty() {
        return;
    }
    block_input.insert(spawn_upgrade_ui.type_id());
    crosshair_state
        .wants_free_cursor
        .insert(spawn_upgrade_ui.type_id());
    let mut menu_commands = commands.spawn((
        ui_root("Upgrade Menu"),
        StateScoped(Screen::Gameplay),
        UpgradeMenu,
    ));
    for upgrade in upgrades.iter() {
        match upgrade {
            Upgrade::Health => {
                menu_commands.with_child(button("Heal", fonts.default.clone(), upgrade_health))
            }
            Upgrade::Damage => menu_commands.with_child(button(
                "Increase Weapon Damage",
                fonts.default.clone(),
                upgrade_damage,
            )),
            Upgrade::Speed => menu_commands.with_child(button(
                "Increase Movement Speed",
                fonts.default.clone(),
                upgrade_speed,
            )),
            Upgrade::Accuracy => menu_commands.with_child(button(
                "Increase Weapon Accuracy",
                fonts.default.clone(),
                upgrade_accuracy,
            )),
            Upgrade::Bullets => menu_commands.with_child(button(
                "More Bullets per Shot",
                fonts.default.clone(),
                upgrade_bullets,
            )),
        };
    }
}

fn hide_upgrade_menu_on_pause(
    mut upgrade_menus: Single<&mut Visibility, With<UpgradeMenu>>,
    pause: Res<State<Pause>>,
) {
    if ***pause {
        **upgrade_menus = Visibility::Hidden;
    } else {
        **upgrade_menus = Visibility::Inherited;
    }
}

fn upgrade_health(
    _: Trigger<Pointer<Click>>,
    mut health: Single<&mut Health, With<Player>>,
    mut commands: Commands,
) {
    health.heal_full();
    commands.trigger(DespawnUpgrades);
}

fn upgrade_speed(
    _: Trigger<Pointer<Click>>,
    mut movement_stats: Single<&mut MovementStats, With<Player>>,
    mut commands: Commands,
) {
    movement_stats.speed_factor += 0.15;
    commands.trigger(DespawnUpgrades);
}

fn upgrade_accuracy(
    _: Trigger<Pointer<Click>>,
    mut weapon_stats: Single<&mut WeaponStats, With<Player>>,
    mut commands: Commands,
) {
    weapon_stats.spread_radius = (weapon_stats.spread_radius - 0.02).max(0.0);
    commands.trigger(DespawnUpgrades);
}

fn upgrade_bullets(
    _: Trigger<Pointer<Click>>,
    mut weapon_stats: Single<&mut WeaponStats, With<Player>>,
    mut commands: Commands,
) {
    weapon_stats.pellets += 4;
    commands.trigger(DespawnUpgrades);
}

fn upgrade_damage(
    _: Trigger<Pointer<Click>>,
    mut weapon_stats: Single<&mut WeaponStats, With<Player>>,
    mut commands: Commands,
) {
    weapon_stats.damage += 1.5;
    commands.trigger(DespawnUpgrades);
}

fn unoffer_upgrades(_trigger: Trigger<WaveFinishedPreparing>, mut commands: Commands) {
    commands.trigger(DespawnUpgrades);
}

fn despawn_upgrades(
    _trigger: Trigger<DespawnUpgrades>,
    mut commands: Commands,
    upgrades: Query<Entity, With<Upgrades>>,
    upgrade_menus: Query<Entity, With<UpgradeMenu>>,
    mut block_input: ResMut<BlocksInput>,
    mut crosshair_state: Single<&mut CrosshairState>,
    mut time: ResMut<Time<Virtual>>,
) {
    for upgrade in upgrades.iter() {
        commands.entity(upgrade).despawn();
    }
    for upgrade_menu in upgrade_menus.iter() {
        commands.entity(upgrade_menu).despawn();
    }
    block_input.remove(&spawn_upgrade_ui.type_id());
    crosshair_state
        .wants_free_cursor
        .remove(&spawn_upgrade_ui.type_id());
    time.unpause();
}

fn pause_in_menu(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

#[derive(Event)]
struct DespawnUpgrades;
