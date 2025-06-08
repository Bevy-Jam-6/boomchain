use std::time::Duration;

use bevy::prelude::*;
use bevy_mesh_decal::Decal;

use crate::PostPhysicsAppSystems;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<DespawnAfter>();
    app.register_type::<Despawn>();

    // This is a weird place for this, but whatever, it's the last day of the jam :P
    app.register_required_components::<Decal, ForceNoOpaque>();

    app.add_systems(
        Update,
        (despawn_after, despawn, fade_out_and_despawn).in_set(PostPhysicsAppSystems::DespawnAfter),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct DespawnAfter(pub(crate) Timer);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct Despawn;

impl DespawnAfter {
    pub(crate) fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

fn despawn_after(
    mut commands: Commands,
    time: Res<Time>,
    mut to_despawn: Query<(&mut DespawnAfter, Entity), Without<Despawn>>,
) {
    for (mut despawn_after, entity) in to_despawn.iter_mut() {
        despawn_after.0.tick(time.delta());
        if despawn_after.0.finished() {
            commands.entity(entity).try_despawn();
        }
    }
}

fn despawn(mut commands: Commands, to_despawn: Query<Entity, With<Despawn>>) {
    for entity in to_despawn.iter() {
        commands.entity(entity).try_despawn();
    }
}

/// A component that can be used to fade out an entity over a specified duration before despawning it.
///
/// Note: This changes the material properties, so it can also fade out unrelaed entities if they share
/// the same material handle.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct FadeOutAndDespawn(pub Timer);

impl FadeOutAndDespawn {
    pub(crate) fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub(crate) struct ForceNoOpaque;

fn fade_out_and_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut to_fade_out: Query<(&mut FadeOutAndDespawn, Entity)>,
    child_query: Query<&Children>,
    mesh_material_query: Query<(&MeshMaterial3d<StandardMaterial>, Has<ForceNoOpaque>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (mut fade_out, entity) in to_fade_out.iter_mut() {
        fade_out.0.tick(time.delta());

        // Fade out the material
        if let Ok((material, _)) = mesh_material_query.get(entity) {
            if let Some(mat) = materials.get_mut(material.id()) {
                mat.base_color.set_alpha(1.0 - fade_out.0.fraction());
                mat.alpha_mode = AlphaMode::Blend;
            }
        }
        for child in child_query.iter_descendants(entity) {
            if let Ok((material, _)) = mesh_material_query.get(child) {
                if let Some(mat) = materials.get_mut(material.id()) {
                    mat.base_color.set_alpha(1.0 - fade_out.0.fraction());
                    mat.alpha_mode = AlphaMode::Blend;
                }
            }
        }

        if fade_out.0.finished() {
            // Set material alphas back to 1.0 before despawning
            if let Ok((material, no_opaque)) = mesh_material_query.get(entity) {
                if let Some(mat) = materials.get_mut(material.id()) {
                    mat.base_color.set_alpha(1.0);
                    if !no_opaque {
                        mat.alpha_mode = AlphaMode::Opaque;
                    } else {
                        mat.alpha_mode = AlphaMode::Mask(0.5);
                    }
                }
            }
            for child in child_query.iter_descendants(entity) {
                if let Ok((material, no_opaque)) = mesh_material_query.get(child) {
                    if let Some(mat) = materials.get_mut(material.id()) {
                        mat.base_color.set_alpha(1.0);
                        if !no_opaque {
                            mat.alpha_mode = AlphaMode::Opaque;
                        } else {
                            mat.alpha_mode = AlphaMode::Mask(0.5);
                        }
                    }
                }
            }

            commands.entity(entity).try_despawn();
        }
    }
}
