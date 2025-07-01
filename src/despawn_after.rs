use std::time::Duration;

use bevy::prelude::*;
use bevy_mesh_decal::Decal;
use rand::Rng as _;

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
pub(crate) struct FadeOutAndDespawn {
    pub(crate) start_fade_out_timer: Timer,
    pub(crate) despawn_timer: Timer,
}

impl FadeOutAndDespawn {
    pub(crate) fn new(duration: Duration) -> Self {
        // Changing the blend mode is expensive, so let's not do it all at once!
        let delay = rand::thread_rng().gen_range(0.0..=2.0);
        let delay = Duration::from_secs_f64(delay);
        Self {
            start_fade_out_timer: Timer::new(delay, TimerMode::Once),
            despawn_timer: Timer::new(duration.saturating_sub(delay), TimerMode::Once),
        }
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
        fade_out.start_fade_out_timer.tick(time.delta());
        if !fade_out.start_fade_out_timer.finished() {
            continue;
        }
        fade_out.despawn_timer.tick(time.delta());

        // Fade out the material
        let alpha = 1.0 - fade_out.despawn_timer.fraction();
        let alpha = alpha * alpha;
        if let Ok((material, _)) = mesh_material_query.get(entity)
            && let Some(mat) = materials.get_mut(material.id())
        {
            mat.base_color.set_alpha(alpha);
            mat.alpha_mode = AlphaMode::Blend;
        }
        for child in child_query.iter_descendants(entity) {
            if let Ok((material, _)) = mesh_material_query.get(child)
                && let Some(mat) = materials.get_mut(material.id())
            {
                mat.base_color.set_alpha(alpha);
                mat.alpha_mode = AlphaMode::Blend;
            }
        }

        if fade_out.despawn_timer.finished() {
            // Set material alphas back to 1.0 before despawning
            if let Ok((material, no_opaque)) = mesh_material_query.get(entity)
                && let Some(mat) = materials.get_mut(material.id())
            {
                mat.base_color.set_alpha(1.0);
                if !no_opaque {
                    mat.alpha_mode = AlphaMode::Opaque;
                } else {
                    mat.alpha_mode = AlphaMode::Mask(0.5);
                }
            }
            for child in child_query.iter_descendants(entity) {
                if let Ok((material, no_opaque)) = mesh_material_query.get(child)
                    && let Some(mat) = materials.get_mut(material.id())
                {
                    mat.base_color.set_alpha(1.0);
                    if !no_opaque {
                        mat.alpha_mode = AlphaMode::Opaque;
                    } else {
                        mat.alpha_mode = AlphaMode::Mask(0.5);
                    }
                }
            }

            commands.entity(entity).try_despawn();
        }
    }
}
