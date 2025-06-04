use bevy::ui::Val::*;
use bevy::{color::palettes::tailwind, prelude::*};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::gameplay::health::Health;
use crate::gameplay::player::Player;
use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_health_bar);
    app.add_systems(Update, update_health_bar);
    app.register_type::<HealthBar>();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct HealthBar;

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
