use bevy::ui::Val::*;
use bevy::{color::palettes::tailwind, prelude::*};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::asset_tracking::LoadResource;
use crate::font::FontAssets;
use crate::gameplay::health::{Health, OnDeath};
use crate::gameplay::npc::Npc;
use crate::gameplay::player::Player;
use crate::gameplay::upgrades::Upgrades;
use crate::gameplay::waves::{WaveAdvanced, WaveFinishedPreparing, WaveStartedPreparing, Waves};
use crate::screens::Screen;
use crate::theme::palette;

pub(super) fn plugin(app: &mut App) {
    app.load_resource::<HudAssets>();
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_health_bar, spawn_wave_hud),
    );
    app.add_systems(
        Update,
        (
            update_health_bar,
            update_prep_time_text,
            update_wave_text,
            blink_upgrade_menu_text,
        ),
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
    health_bar_texture: Handle<Image>,
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
            health_bar_texture: assets.load({
                #[cfg(feature = "dev")]
                {
                    "images/blood/BloodFabricHealthBarGrayscale.png"
                }
                #[cfg(not(feature = "dev"))]
                {
                    "images/blood/BloodFabricHealthBarGrayscale.ktx2"
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

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
pub(crate) struct UpgradeMenuText(Timer);

fn spawn_wave_hud(mut commands: Commands, fonts: Res<FontAssets>) {
    commands.spawn((
        Name::new("Spawn Wave HUD"),
        Node {
            flex_direction: FlexDirection::Column,
            margin: UiRect::horizontal(Auto),
            align_items: AlignItems::Center,
            top: Px(10.0),
            row_gap: Px(5.0),
            ..default()
        },
        StateScoped(Screen::Gameplay),
        Pickable::IGNORE,
        children![
            (
                Text::new("Wave 1/10:"),
                TextFont::from_font_size(26.0).with_font(fonts.default.clone()),
                WaveText
            ),
            (
                Node {
                    width: Percent(300.0),
                    max_width: Px(1200.0),
                    min_height: Px(32.0),
                    margin: UiRect::horizontal(Px(10.0)),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                },
                BorderRadius::all(Px(10.0)),
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
            align_content: AlignContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        ImageNode::new(hud_assets.angry.clone()).with_color(palette::LABEL_TEXT),
        AngryIcon(enemy),
    ));
}

fn add_dead_icon(
    trigger: Trigger<OnDeath>,
    enemies: Query<(), With<Npc>>,
    container: Single<Entity, With<WaveIconParent>>,
    children: Query<&Children>,
    angry_icon: Query<&AngryIcon>,
    mut image_node: Query<&mut ImageNode>,
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
    let Ok(mut image_node) = image_node.get_mut(angry_icon) else {
        error!("No `ImageNode` found for angry icon {angry_icon:?}");
        return;
    };

    // Make the angry icon dimmer.
    image_node.color = image_node.color.with_alpha(0.5);
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
    fonts: Res<FontAssets>,
    mut commands: Commands,
) {
    commands.entity(*container).with_children(|parent| {
        parent.spawn((
            Node {
                margin: UiRect::horizontal(Px(10.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            children![
                (
                    Text::new(""),
                    TextFont::from_font(fonts.default.clone()).with_font_size(18.0),
                    TextColor(palette::LABEL_TEXT),
                    PrepTimeText
                ),
                (
                    Node {
                        margin: UiRect::top(Px(10.0)),
                        ..default()
                    },
                    Text::new("Press F to upgrade!"),
                    TextFont::default()
                        .with_font_size(26.0)
                        .with_font(fonts.default.clone()),
                    TextColor(Color::from(tailwind::GREEN_500)),
                    UpgradeMenuText(Timer::from_seconds(0.7, TimerMode::Once)),
                ),
            ],
        ));
    });
}

fn blink_upgrade_menu_text(
    upgrade_menu_text: Single<(Entity, &mut UpgradeMenuText, &mut Visibility)>,
    upgrades: Query<(), With<Upgrades>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let (entity, mut timer, mut visibility) = upgrade_menu_text.into_inner();
    if upgrades.is_empty() {
        commands.entity(entity).despawn();
    }
    timer.tick(time.delta());
    if timer.finished() {
        *visibility = match *visibility {
            Visibility::Visible | Visibility::Inherited => Visibility::Hidden,
            Visibility::Hidden => Visibility::Inherited,
        };
        **timer = match *visibility {
            Visibility::Visible | Visibility::Inherited => {
                Timer::from_seconds(0.7, TimerMode::Once)
            }
            Visibility::Hidden => Timer::from_seconds(0.3, TimerMode::Once),
        }
    }
}

fn update_prep_time_text(
    waves: Single<&Waves>,
    mut prep_time_text: Single<&mut Text, With<PrepTimeText>>,
) {
    ***prep_time_text = format!(
        "Next wave in {} s",
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
fn spawn_health_bar(
    health: Single<&Health, With<Player>>,
    hud_assets: Res<HudAssets>,
    mut commands: Commands,
) {
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
            bottom: Px(20.0),
            ..default()
        },
        Pickable::IGNORE,
        children![(
            Node {
                width: Percent(100.0),
                max_width: Px(500.0),
                height: Px(15.0),
                ..default()
            },
            BorderRadius::MAX,
            BackgroundColor(Color::from(tailwind::ZINC_900.with_alpha(0.8))),
            children![(
                HealthBar,
                Node {
                    width: Percent(hp * 100.0),
                    height: Percent(100.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderRadius::all(Px(10.0)),
                BackgroundColor(Color::from(tailwind::RED_600.with_alpha(0.5))),
                children![(
                    ImageNode {
                        color: tailwind::RED_400.with_alpha(0.75).into(),
                        image: hud_assets.health_bar_texture.clone(),
                        image_mode: NodeImageMode::Auto,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        top: Px(-250.0),
                        left: Px(0.0),
                        width: Px(500.0),
                        height: Px(500.0),
                        ..default()
                    },
                ),]
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
