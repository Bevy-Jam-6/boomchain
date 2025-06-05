//! [bevy_mesh_decal](https://github.com/Aronry/bevy_mesh_decal) is our CPU decal system.
//! It supports both native and web builds, unlike Bevy's current built-in decals.

use avian3d::prelude::Collider;
use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_mesh_decal::{DecalPlugin, Decalable};
use bevy_trenchbroom::geometry::MapGeometry;

use crate::gameplay::npc::{Npc, lifecycle::Gib};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(DecalPlugin);
    app.register_required_components::<MapGeometry, Decalable>();
    app.register_required_components::<Collider, Decalable>();
    app.add_observer(
        |trigger: Trigger<SceneInstanceReady>,
         children: Query<&Children, Or<(With<Npc>, With<Gib>)>>,
         mut commands: Commands| {
            let batch = children
                .iter_descendants(trigger.target())
                .map(|e| (e, Decalable::default()))
                .collect::<Vec<_>>();
            commands.insert_batch(batch);
        },
    );
}
