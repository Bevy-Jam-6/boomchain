use crate::asset_tracking::LoadResource as _;
use crate::gameplay::explosion::ExplodeOnShoot;
use crate::gameplay::health::OnDamage;
use crate::gameplay::npc::ai_state::AiState;
use crate::gameplay::player::Player;
use crate::gameplay::player::camera::PlayerCamera;
use crate::gameplay::player::camera_shake::NonTraumaTransform;
use crate::gameplay::player::gunplay::Shooting;
use crate::screens::loading::LoadingScreen;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::render::render_resource::{CachedPipelineState, PipelineCache};
use bevy::render::{MainWorld, RenderApp};
use bevy_enhanced_input::prelude::*;
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;
use std::time::Duration;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<CompileShadersAssets>();

    app.init_resource::<LoadedPipelineCount>();
    app.init_resource::<PipelinesReady>();

    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, update_loaded_pipeline_count);

    app.add_systems(
        Update,
        (explode_enemy, explode_barrel)
            .run_if(in_state(LoadingScreen::Shaders))
            .run_if(any_with_component::<Camera3d>),
    );
    app.add_systems(
        PreUpdate,
        shoot
            .before(EnhancedInputSystem)
            .run_if(in_state(LoadingScreen::Shaders))
            .run_if(any_with_component::<Camera3d>),
    );
    app.add_systems(
        PostUpdate,
        look_ahead.run_if(in_state(LoadingScreen::Shaders)),
    );
    app.add_systems(
        Update,
        force_proceed.run_if(in_state(LoadingScreen::Shaders)),
    );

    app.register_type::<LoadedPipelineCount>();
}

#[cfg_attr(feature = "hot_patch", hot)]
pub(crate) fn spawn_shader_compilation_map(
    mut commands: Commands,
    compile_shaders_assets: Res<CompileShadersAssets>,
) {
    commands.spawn((
        Name::new("Compile Shaders Map"),
        SceneRoot(compile_shaders_assets.level.clone()),
        StateScoped(LoadingScreen::Shaders),
    ));

    commands.spawn((
        StateScoped(LoadingScreen::Shaders),
        DirectionalLight {
            illuminance: 5_000.0,
            color: Color::srgb_u8(200, 190, 255),
            shadows_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            maximum_distance: 400.0,
            first_cascade_far_bound: 40.0,
            ..default()
        }
        .build(),
        Transform::default().looking_to(Vec3::new(-1.75, -1.0, 0.5), Vec3::Y),
    ));
}

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct CompileShadersAssets {
    #[dependency]
    level: Handle<Scene>,
}

impl FromWorld for CompileShadersAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();

        Self {
            // This map just loads all effects at once to try to force shader compilation.
            level: assets.load("maps/compile_shaders/compile_shaders.map#Scene"),
        }
    }
}

/// A `Resource` in the main world that stores the number of pipelines that are ready.
#[derive(Resource, Default, Debug, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub(crate) struct LoadedPipelineCount(pub(crate) usize);

#[derive(Resource, Default, Debug, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub(crate) struct PipelinesReady(pub(crate) bool);

impl LoadedPipelineCount {
    pub(crate) fn is_done(&self) -> bool {
        self.0 >= Self::TOTAL_PIPELINES
    }

    /// These numbers have to be tuned by hand, unfortunately.
    /// When in doubt, better stay a bit too low, or the player won't advance past the loading screen.
    pub(crate) const TOTAL_PIPELINES: usize = {
        #[cfg(feature = "native")]
        {
            #[cfg(feature = "dev")]
            {
                74
            }
            #[cfg(not(feature = "dev"))]
            {
                73
            }
        }
        #[cfg(not(feature = "native"))]
        {
            #[cfg(feature = "dev")]
            {
                47
            }
            #[cfg(not(feature = "dev"))]
            {
                46
            }
        }
    };
}

fn force_proceed(
    mut loaded_pipeline_count: ResMut<LoadedPipelineCount>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let timer = timer.get_or_insert_with(|| Timer::new(Duration::from_secs(60), TimerMode::Once));
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }
    loaded_pipeline_count.0 = 9999;
}

fn look_ahead(mut cameras: Query<(&mut Transform, &mut NonTraumaTransform), With<PlayerCamera>>) {
    for (mut camera, mut non_trauma_transform) in &mut cameras {
        camera.look_to(Vec3::NEG_Z, Vec3::Y);
        non_trauma_transform.0 = *camera;
    }
}

fn explode_enemy(
    enemies: Query<Entity, With<AiState>>,
    mut commands: Commands,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_millis(500), TimerMode::Once));
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }
    for entity in &enemies {
        commands.entity(entity).trigger(OnDamage(1000.0));
    }
}

fn explode_barrel(
    barrels: Query<Entity, With<ExplodeOnShoot>>,
    mut commands: Commands,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_millis(500), TimerMode::Once));
    timer.tick(time.delta());
    if !timer.finished() {
        return;
    }
    for entity in &barrels {
        commands.entity(entity).trigger(OnDamage(1000.0));
    }
}

fn shoot(
    players: Query<Entity, With<Player>>,
    mut commands: Commands,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_millis(1000), TimerMode::Once));
    timer.tick(time.delta());
    if timer.finished() {
        return;
    }
    for player in &players {
        commands.entity(player).insert(Shooting);
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn update_loaded_pipeline_count(mut main_world: ResMut<MainWorld>, cache: Res<PipelineCache>) {
    if let Some(mut pipelines_ready) = main_world.get_resource_mut::<LoadedPipelineCount>() {
        let count = cache
            .pipelines()
            .filter(|pipeline| matches!(pipeline.state, CachedPipelineState::Ok(_)))
            .count();

        if pipelines_ready.0 >= count {
            return;
        }
        info!("loaded {count} pipelines");

        pipelines_ready.0 = count;
    }
    if let Some(mut pipelines_ready) = main_world.get_resource_mut::<PipelinesReady>() {
        pipelines_ready.0 = cache.waiting_pipelines().count() == 0;
    }
}

pub(crate) fn all_pipelines_loaded(
    loaded_pipeline_count: Res<LoadedPipelineCount>,
    // Sometimes not correctly picked up, so eh, let's remove that
    // pipelines_ready: Res<PipelinesReady>,
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
) -> bool {
    let is_done_but_still_playing_particles = loaded_pipeline_count.is_done();
    if !is_done_but_still_playing_particles {
        return false;
    }
    // Need some time to let the particles finish playing
    let timer =
        timer.get_or_insert_with(|| Timer::new(Duration::from_millis(1500), TimerMode::Once));
    timer.tick(time.delta());
    if !timer.finished() {
        return false;
    }
    true
}
