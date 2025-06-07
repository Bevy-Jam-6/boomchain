use std::time::Duration;

use bevy::prelude::*;
use bevy_mesh_decal::Decal;

use crate::{
    despawn_after::DespawnAfter, gameplay::npc::lifecycle::Gib, menus::Menu, screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GoreSettings>();
    app.init_resource::<GoreSettings>();

    app.add_systems(
        Update,
        (despawn_decals, despawn_gibs)
            .run_if(in_state(Screen::Gameplay).and(not(in_state(Menu::Pause)))),
    );
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub(crate) struct GoreSettings {
    pub blood_decals: Gore,
    pub gibs: Gore,
}

impl Default for GoreSettings {
    fn default() -> Self {
        Self {
            blood_decals: Gore::Despawn(Duration::from_secs(10)),
            gibs: Gore::Despawn(Duration::from_secs(10)),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gore {
    None,
    NeverDespawn,
    Despawn(Duration),
}

fn despawn_decals(
    mut commands: Commands,
    decals: Query<(Entity, Ref<Decal>)>,
    decals_with_despawn_timer: Query<Entity, (With<Decal>, With<DespawnAfter>)>,
    gore_settings: Res<GoreSettings>,
) {
    match gore_settings.blood_decals {
        Gore::None => {
            for (entity, _) in &decals {
                commands.entity(entity).despawn();
            }
        }
        Gore::NeverDespawn => {
            for entity in &decals_with_despawn_timer {
                commands.entity(entity).remove::<DespawnAfter>();
            }
        }
        Gore::Despawn(duration) => {
            let setting_changed = gore_settings.is_changed();
            for (entity, decal) in &decals {
                if setting_changed || decal.is_added() {
                    commands
                        .entity(entity)
                        .insert_if_new(DespawnAfter(Timer::new(duration, TimerMode::Once)));
                }
            }
        }
    }
}

fn despawn_gibs(
    mut commands: Commands,
    gibs: Query<(Entity, Ref<Gib>)>,
    gibs_with_despawn_timer: Query<Entity, (With<Gib>, With<DespawnAfter>)>,
    gore_settings: Res<GoreSettings>,
) {
    match gore_settings.gibs {
        Gore::None => {
            for (entity, _) in &gibs {
                commands.entity(entity).despawn();
            }
        }
        Gore::NeverDespawn => {
            for entity in &gibs_with_despawn_timer {
                commands.entity(entity).remove::<DespawnAfter>();
            }
        }
        Gore::Despawn(duration) => {
            let setting_changed = gore_settings.is_changed();
            for (entity, gib) in &gibs {
                if setting_changed || gib.is_added() {
                    commands
                        .entity(entity)
                        .insert_if_new(DespawnAfter(Timer::new(duration, TimerMode::Once)));
                }
            }
        }
    }
}
