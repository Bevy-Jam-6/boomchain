use bevy::ui::Val::*;
use bevy::{color::palettes::tailwind, prelude::*};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::asset_tracking::LoadResource;
use crate::gameplay::health::{Health, OnDeath};
use crate::gameplay::npc::Npc;
use crate::gameplay::player::Player;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<HudAssets>();
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_health_bar, spawn_wave_hud),
    );
    app.add_systems(Update, update_health_bar);
    app.register_type::<HealthBar>();
    app.register_type::<WaveText>();
    app.add_observer(add_angry_icon);
    app.add_observer(add_dead_icon);
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
pub(crate) struct AngryIcon;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct DeadIcon;

fn spawn_wave_hud(mut commands: Commands) {
    commands.spawn((
        Name::new("Health HUD"),
        GlobalZIndex(2),
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
    _trigger: Trigger<OnAdd, Npc>,
    container: Single<Entity, With<WaveIconParent>>,
    mut commands: Commands,
    hud_assets: Res<HudAssets>,
) {
    commands.entity(*container).with_child((
        Node {
            width: Px(32.0),
            height: Px(32.0),
            ..default()
        },
        ImageNode::new(hud_assets.angry.clone()).with_color(Color::BLACK),
        AngryIcon,
    ));
}

fn add_dead_icon(
    trigger: Trigger<OnDeath>,
    enemies: Query<(), With<Npc>>,
    container: Single<Entity, With<WaveIconParent>>,
    mut commands: Commands,
    children: Query<&Children>,
    angry_icon: Query<Entity, With<AngryIcon>>,
    hud_assets: Res<HudAssets>,
) {
    let entity = trigger.target();
    if !enemies.contains(entity) {
        return;
    }
    let Ok(icons) = children.get(*container) else {
        return;
    };
    let Some(first_angry_icon) = angry_icon.iter().find(|icon| icons.contains(icon)) else {
        return;
    };

    commands
        .entity(first_angry_icon)
        .remove::<AngryIcon>()
        .insert(DeadIcon)
        .with_child((
            ImageNode::new(hud_assets.dead.clone()).with_color(Color::srgba(1.0, 0.0, 0.0, 1.0)),
        ));
}

#[cfg_attr(feature = "hot_patch", hot(rerun_on_hot_patch = true))]
fn spawn_health_bar(health: Single<&Health, With<Player>>, mut commands: Commands) {
    let hp = health.fraction();
    commands.spawn((
        Name::new("Health HUD"),
        StateScoped(Screen::Gameplay),
        GlobalZIndex(2),
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
    health: Single<&Health, With<Player>>,
    mut health_bar: Single<&mut Node, With<HealthBar>>,
) {
    let hp = health.fraction();
    health_bar.width = Percent(hp * 100.0);
}
