use bevy::ui::Val::*;
use bevy::{color::palettes::tailwind, prelude::*};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::asset_tracking::LoadResource;
use crate::gameplay::health::{Health, OnDeath};
use crate::gameplay::npc::Npc;
use crate::gameplay::player::Player;
use crate::gameplay::waves::{WaveAdvanced, WaveFinishedPreparing, WaveStartedPreparing, Waves};
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<HudAssets>();
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_health_bar, spawn_wave_hud),
    );
    app.add_systems(
        Update,
        (update_health_bar, update_prep_time_text, update_wave_text),
    );
    app.register_type::<HealthBar>();
    app.register_type::<WaveText>();
    app.add_observer(add_angry_icon);
    app.add_observer(add_dead_icon);
    app.add_observer(flush_on_wave_advanced);
    app.add_observer(spawn_prep_icon);
    app.add_observer(flush_on_prep_time_finished);
}

#[derive(Resource, Asset, Clone, Reflect)]
struct HudAssets {
    #[dependency]
    angry: Handle<Image>,
    #[dependency]
    dead: Handle<Image>,
}

impl FromWorld for HudAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            angry: assets.load({
                #[cfg(feature = "dev")]
                {
                    "ui/angry.png"
                }
                #[cfg(not(feature = "dev"))]
                {
                    "ui/angry.ktx2"
                }
            }),
            dead: assets.load({
                #[cfg(feature = "dev")]
                {
                    "ui/dead.png"
                }
                #[cfg(not(feature = "dev"))]
                {
                    "ui/dead.ktx2"
                }
            }),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct HealthBar;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct WaveText;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct WaveIconParent;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct AngryIcon(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct WaveInProgressIcon;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct DeadIcon;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct PrepTimeText;

fn spawn_wave_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Health HUD"),
        Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::horizontal(Auto),
            align_items: AlignItems::Center,
            top: Px(20.0),
            ..default()
        },
        StateScoped(Screen::Gameplay),
        Pickable::IGNORE,
        children![
            (Text::new("Wave 1/10:"), WaveText),
            (
                Node {
                    width: Percent(100.0),
                    max_width: Px(300.0),
                    min_height: Px(50.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                BorderRadius::all(Px(10.0)),
                BackgroundColor(Color::from(tailwind::GRAY_200).with_alpha(0.3)),
                WaveIconParent,
            )
        ],
    ));
}

fn add_angry_icon(
    trigger: Trigger<OnAdd, Npc>,
    container: Single<(Entity, Option<&Children>), With<WaveIconParent>>,
    mut commands: Commands,
    hud_assets: Res<HudAssets>,
    angry_icons: Query<&AngryIcon>,
) {
    let enemy = trigger.target();
    let (container, children) = container.into_inner();
    if children.is_some_and(|children| {
        children
            .iter()
            .any(|child| angry_icons.get(child).is_ok_and(|icon| icon.0 == enemy))
    }) {
        warn!("Angry icon already exists for enemy {enemy}");
        return;
    }
    commands.entity(container).with_child((
        Node {
            width: Px(32.0),
            height: Px(32.0),
            ..default()
        },
        ImageNode::new(hud_assets.angry.clone()).with_color(Color::srgba(0.0, 0.0, 0.0, 3.0)),
        AngryIcon(enemy),
    ));
}

fn add_dead_icon(
    trigger: Trigger<OnDeath>,
    enemies: Query<(), With<Npc>>,
    container: Single<Entity, With<WaveIconParent>>,
    mut commands: Commands,
    children: Query<&Children>,
    angry_icon: Query<&AngryIcon>,
    hud_assets: Res<HudAssets>,
) {
    let entity = trigger.target();
    if !enemies.contains(entity) {
        return;
    }
    let Ok(icons) = children.get(*container) else {
        error!("No children found for container");
        return;
    };
    let Some(angry_icon) = icons
        .iter()
        .find(|child| angry_icon.get(*child).is_ok_and(|icon| icon.0 == entity))
    else {
        error!("No angry icon found for entity {entity}");
        return;
    };

    commands
        .entity(angry_icon)
        .remove::<AngryIcon>()
        .insert(DeadIcon)
        .with_child((
            ImageNode::new(hud_assets.dead.clone()).with_color(Color::srgba(1.0, 0.0, 0.0, 3.0)),
        ));
}

fn flush_on_wave_advanced(
    _trigger: Trigger<WaveAdvanced>,
    container: Single<Entity, With<WaveIconParent>>,
    mut commands: Commands,
) {
    commands.entity(*container).despawn_related::<Children>();
}

fn update_wave_text(waves: Single<&Waves>, mut wave_text: Single<&mut Text, With<WaveText>>) {
    ***wave_text = format!(
        "Wave {}/{}",
        waves.current_wave_index() + 1,
        waves.total_waves()
    );
}

fn spawn_prep_icon(
    _trigger: Trigger<WaveStartedPreparing>,
    container: Single<Entity, With<WaveIconParent>>,
    mut commands: Commands,
) {
    commands.entity(*container).with_child((
        Node {
            margin: UiRect::horizontal(Px(10.0)),
            ..default()
        },
        Text::new(""),
        PrepTimeText,
    ));
}

fn update_prep_time_text(
    waves: Single<&Waves>,
    mut prep_time_text: Single<&mut Text, With<PrepTimeText>>,
) {
    ***prep_time_text = format!(
        "Get Ready! {} s",
        waves.prep_time_left().as_secs_f32().ceil() as u32
    );
}

fn flush_on_prep_time_finished(
    _trigger: Trigger<WaveFinishedPreparing>,
    container: Single<Entity, With<WaveIconParent>>,
    mut commands: Commands,
) {
    commands.entity(*container).despawn_related::<Children>();
}

#[cfg_attr(feature = "hot_patch", hot(rerun_on_hot_patch = true))]
fn spawn_health_bar(health: Single<&Health, With<Player>>, mut commands: Commands) {
    let hp = health.fraction();
    commands.spawn((
        Name::new("Health HUD"),
        StateScoped(Screen::Gameplay),
        Node {
            position_type: PositionType::Absolute,
            width: Percent(100.0),
            height: Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            bottom: Px(60.0),
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Node {
                width: Percent(100.0),
                max_width: Px(500.0),
                height: Px(50.0),
                ..default()
            },
            BorderRadius::all(Px(10.0)),
            BackgroundColor(Color::from(tailwind::GRAY_600)),
            children![(
                HealthBar,
                Node {
                    width: Percent(hp * 100.0),
                    height: Percent(100.0),
                    ..default()
                },
                BorderRadius::all(Px(10.0)),
                BackgroundColor(Color::from(tailwind::RED_500)),
            )],
        )],
    ));
}

fn update_health_bar(
    health: Single<Option<&Health>, With<Player>>,
    mut health_bar: Single<&mut Node, With<HealthBar>>,
) {
    let hp = health.map(|h| h.fraction()).unwrap_or(0.0);
    health_bar.width = Percent(hp * 100.0);
}
