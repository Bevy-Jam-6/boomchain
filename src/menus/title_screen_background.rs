//! The animated background for the title screen.

use bevy::{
    color::palettes::tailwind::{GRAY_600, RED_600},
    prelude::*,
    window::PrimaryWindow,
};
use rand::Rng;

use crate::{menus::assets::MenuAssets, screens::Screen, theme::palette::SCREEN_BACKGROUND};

const MIN_BACKGROUND_ALPHA: f32 = 0.1;
const MAX_BACKGROUND_ALPHA: f32 = 0.6;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), spawn_title_screen_background);
    app.add_systems(
        Update,
        (scroll_background, animate_alpha).run_if(in_state(Screen::Title)),
    );
}

#[derive(Component)]
struct AlphaAnimationPhaseProgress(f32);

fn spawn_title_screen_background(mut commands: Commands, menu_assets: Res<MenuAssets>) {
    let mut rng = rand::thread_rng();

    let background_entity = commands
        .spawn((
            Name::new("Title Screen Background"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(SCREEN_BACKGROUND),
            StateScoped(Screen::Title),
        ))
        .id();

    for row in -4..5 {
        for col in -2..5 {
            let x_percentage_offset = rng.r#gen::<f32>() * 25.0;
            let y_percentage_offset = rng.r#gen::<f32>() * 25.0;
            let size_percentage_offset = rng.r#gen::<f32>() * 25.0;
            let rotation = Quat::from_rotation_z(rng.r#gen::<f32>() * std::f32::consts::TAU);
            let t = rng.r#gen::<f32>();
            let alpha = MIN_BACKGROUND_ALPHA.lerp(MAX_BACKGROUND_ALPHA, t);
            commands.spawn((
                ImageNode {
                    color: GRAY_600
                        .mix(&RED_600, rng.r#gen::<f32>())
                        .with_alpha(alpha)
                        .into(),
                    image: menu_assets.background_texture_1.clone(),
                    image_mode: NodeImageMode::Auto,
                    ..default()
                },
                AlphaAnimationPhaseProgress(t),
                ChildOf(background_entity),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(100.0 / 5.0 * row as f32 + y_percentage_offset),
                    left: Val::Percent(100.0 / 5.0 * col as f32 + x_percentage_offset),
                    width: Val::Percent(25.0 + size_percentage_offset),
                    aspect_ratio: Some(1.0),
                    ..default()
                },
                Transform::from_rotation(rotation),
            ));
        }
    }
}

fn animate_alpha(
    time: Res<Time>,
    mut query: Query<(&mut ImageNode, &AlphaAnimationPhaseProgress)>,
) {
    for (mut node, phase) in &mut query {
        let phase = (2.0 * phase.0 - 1.0).asin();
        let mut t = (time.elapsed_secs() + phase).sin() * 0.5 + 0.5; // Normalize to [0, 1]
        t = MIN_BACKGROUND_ALPHA + (MAX_BACKGROUND_ALPHA - MIN_BACKGROUND_ALPHA) * t;
        node.color.set_alpha(t);
    }
}

fn scroll_background(
    time: Res<Time>,
    mut query: Query<(&mut Node, &ComputedNode), With<ImageNode>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let window_height = window.height();
    for (mut node, computed_node) in &mut query {
        if let Val::Px(top) = &mut node.top {
            *top += time.delta_secs() * 40.0;
            if *top > window_height {
                *top = -computed_node.size.y;
            }
        } else {
            // Convert to pixels if it's a percentage
            if let Val::Percent(percent) = &node.top {
                let new_top = percent * window_height / 100.0 + time.delta_secs() * 40.0;
                node.top = Val::Px(new_top);
                if new_top > window_height {
                    node.top = Val::Px(-computed_node.size.y);
                }
            }
        }
    }
}
