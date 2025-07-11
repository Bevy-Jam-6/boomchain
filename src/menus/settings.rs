//! The settings screen accessible from the title screen.
//! We can add all manner of settings and accessibility options here.
//! For 3D, we'd also place the camera sensitivity and FOV here.

use std::time::Duration;

use bevy::{
    audio::Volume, ecs::spawn::SpawnWith, input::common_conditions::input_just_pressed, prelude::*,
    ui::Val::*,
};
#[cfg(feature = "hot_patch")]
use bevy_simple_subsecond_system::hot;

use crate::{
    audio::{DEFAULT_VOLUME, max_volume},
    font::FontAssets,
    gameplay::{
        gore_settings::{Gore, GoreSettings},
        player::camera::{CameraSensitivity, MouseInversion, WorldModelFov},
    },
    menus::Menu,
    screens::Screen,
    theme::{prelude::*, widget::OnChangeSelection},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<VolumeSliderSettings>();
    app.add_systems(OnEnter(Menu::Settings), spawn_settings_menu);
    app.add_systems(
        Update,
        go_back.run_if(in_state(Menu::Settings).and(input_just_pressed(KeyCode::Escape))),
    );

    app.register_type::<GlobalVolumeLabel>();
    app.add_systems(
        Update,
        (
            update_global_volume.run_if(resource_exists_and_changed::<VolumeSliderSettings>),
            update_volume_label,
            update_camera_sensitivity_label,
            update_camera_fov_label,
            update_gib_count_label,
        )
            .run_if(in_state(Menu::Settings)),
    );
}

#[cfg_attr(feature = "hot_patch", hot)]
fn spawn_settings_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    gore_settings: Res<GoreSettings>,
    mouse_inversion: Res<MouseInversion>,
) {
    let fonts_outer = fonts.clone();
    let fonts = fonts.clone();
    let gore_settings = gore_settings.clone();
    let mouse_inversion = mouse_inversion.clone();
    commands.spawn((
        widget::ui_root("Settings Screen"),
        StateScoped(Menu::Settings),
        GlobalZIndex(2),
        children![
            widget::header("Settings", fonts.default.clone()),
            (
                Name::new("Settings Grid"),
                Node {
                    display: Display::Grid,
                    row_gap: Px(10.0),
                    column_gap: Px(30.0),
                    grid_template_columns: RepeatedGridTrack::px(2, 400.0),
                    ..default()
                },
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    parent.spawn(
                        // Audio
                        (
                            widget::label("Audio Volume", fonts.default.clone()),
                            Node {
                                justify_self: JustifySelf::End,
                                ..default()
                            },
                        ),
                    );
                    parent.spawn(widget::plus_minus_bar(
                        GlobalVolumeLabel,
                        lower_volume,
                        raise_volume,
                        fonts.default.clone(),
                        Handle::<Font>::default(),
                    ));
                    // Camera Sensitivity
                    parent.spawn((
                        widget::label("Camera Sensitivity", fonts.default.clone()),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        },
                    ));
                    parent.spawn(widget::plus_minus_bar(
                        CameraSensitivityLabel,
                        lower_camera_sensitivity,
                        raise_camera_sensitivity,
                        fonts.default.clone(),
                        fonts.default.clone(),
                    ));
                    // Invert mouse Y
                    parent.spawn((
                        widget::label("Invert Mouse Y", fonts.default.clone()),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        },
                    ));
                    parent.spawn(widget::cycle_select(
                        vec!["No".to_string(), "Yes".to_string()],
                        if mouse_inversion.invert_mouse_y { 1 } else { 0 },
                        fonts.default.clone(),
                        |trigger: Trigger<OnChangeSelection>,
                         mut mouse_inversion: ResMut<MouseInversion>| {
                            let selection = trigger.selection;
                            mouse_inversion.invert_mouse_y = selection == 1;
                        },
                    ));
                    // Camera FOV
                    parent.spawn((
                        widget::label("Camera FOV", fonts.default.clone()),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        },
                    ));
                    parent.spawn(widget::plus_minus_bar(
                        CameraFovLabel,
                        lower_camera_fov,
                        raise_camera_fov,
                        fonts.default.clone(),
                        fonts.default.clone(),
                    ));
                    // Gib count
                    parent.spawn((
                        widget::label("Number of body parts", fonts.default.clone()),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        },
                    ));
                    parent.spawn(widget::plus_minus_bar(
                        GibCountLabel,
                        lower_gib_count,
                        raise_gib_count,
                        fonts.default.clone(),
                        fonts.default.clone(),
                    ));
                    // Gore settings for dismemberment
                    parent.spawn((
                        widget::label("Dismemberment", fonts.default.clone()),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        },
                    ));
                    parent.spawn(widget::cycle_select(
                        vec![
                            "Enabled (despawn after waves)".to_string(),
                            "Enabled (despawn after 10 s)".to_string(),
                            "Enabled (never despawn)".to_string(),
                            "Disabled".to_string(),
                        ],
                        match gore_settings.blood_decals {
                            Gore::DespawnAfterWave => 0,
                            Gore::Despawn(_) => 1,
                            Gore::NeverDespawn => 2,
                            Gore::None => 3,
                        },
                        fonts.default.clone(),
                        |trigger: Trigger<OnChangeSelection>,
                         mut gore_settings: ResMut<GoreSettings>| {
                            let selection = trigger.selection;
                            gore_settings.gibs = match selection {
                                0 => Gore::DespawnAfterWave,
                                1 => Gore::Despawn(Duration::from_secs(10)),
                                2 => Gore::NeverDespawn,
                                _ => Gore::None,
                            };
                        },
                    ));
                    // Gore settings for blood decals
                    parent.spawn((
                        widget::label("Blood Splatter", fonts.default.clone()),
                        Node {
                            justify_self: JustifySelf::End,
                            ..default()
                        },
                    ));
                    parent.spawn(widget::cycle_select(
                        vec![
                            "Enabled (despawn after waves)".to_string(),
                            "Enabled (despawn after 10 s)".to_string(),
                            "Enabled (never despawn)".to_string(),
                            "Disabled".to_string(),
                        ],
                        match gore_settings.blood_decals {
                            Gore::DespawnAfterWave => 0,
                            Gore::Despawn(_) => 1,
                            Gore::NeverDespawn => 2,
                            Gore::None => 3,
                        },
                        fonts.default.clone(),
                        |trigger: Trigger<OnChangeSelection>,
                         mut gore_settings: ResMut<GoreSettings>| {
                            let selection = trigger.selection;
                            gore_settings.blood_decals = match selection {
                                0 => Gore::DespawnAfterWave,
                                1 => Gore::Despawn(Duration::from_secs(10)),
                                2 => Gore::NeverDespawn,
                                _ => Gore::None,
                            };
                        },
                    ));
                })),
            ),
            widget::button("Back", fonts_outer.default.clone(), go_back_on_click),
        ],
    ));
}

#[derive(Resource, Reflect, Debug)]
struct VolumeSliderSettings(usize);

impl VolumeSliderSettings {
    fn increment(&mut self) {
        self.0 = Self::MAX_TICK_COUNT.min(self.0 + 1);
    }

    fn decrement(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    fn volume(&self) -> Volume {
        let max_gain = max_volume().to_linear();
        let mid_gain = DEFAULT_VOLUME.to_linear();

        let t = self.0 as f32 / Self::MAX_TICK_COUNT as f32;
        let gain = Self::curved_interpolation(t, mid_gain, max_gain);
        Volume::Linear(gain)
    }

    /// Interpolates between 0, a, and b nonlinearly,
    /// such that t = 0 -> 0, t = 0.5 -> a, t = 1 -> b
    fn curved_interpolation(t: f32, a: f32, b: f32) -> f32 {
        if t <= 0.5 {
            let t2 = t / 0.5;
            a * (3.0 * t2.powi(2) - 2.0 * t2.powi(3))
        } else {
            let t2 = (t - 0.5) / 0.5;
            let smooth = 3.0 * t2.powi(2) - 2.0 * t2.powi(3);
            a + (b - a) * smooth
        }
    }

    /// How many ticks the volume slider supports
    const MAX_TICK_COUNT: usize = 20;
}

impl Default for VolumeSliderSettings {
    fn default() -> Self {
        Self(Self::MAX_TICK_COUNT / 2)
    }
}

#[cfg_attr(feature = "hot_patch", hot)]
fn update_global_volume(
    mut global_volume: ResMut<GlobalVolume>,
    volume_step: Res<VolumeSliderSettings>,
) {
    global_volume.volume = volume_step.volume();
}

#[cfg_attr(feature = "hot_patch", hot)]
fn lower_volume(_trigger: Trigger<Pointer<Click>>, mut volume_step: ResMut<VolumeSliderSettings>) {
    volume_step.decrement();
}

#[cfg_attr(feature = "hot_patch", hot)]
fn raise_volume(_trigger: Trigger<Pointer<Click>>, mut volume_step: ResMut<VolumeSliderSettings>) {
    volume_step.increment();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GlobalVolumeLabel;

#[cfg_attr(feature = "hot_patch", hot)]
fn update_volume_label(
    mut label: Single<&mut Text, With<GlobalVolumeLabel>>,
    slider: Res<VolumeSliderSettings>,
) {
    let ticks = slider.0;
    let filled = "█".repeat(ticks);
    let empty = " ".repeat(VolumeSliderSettings::MAX_TICK_COUNT - ticks);
    let text = filled + &empty + "|";
    label.0 = text;
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct CameraSensitivityLabel;

#[cfg_attr(feature = "hot_patch", hot)]
fn lower_camera_sensitivity(
    _trigger: Trigger<Pointer<Click>>,
    mut camera_sensitivity: ResMut<CameraSensitivity>,
) {
    camera_sensitivity.0 -= 0.1;
    const MIN_SENSITIVITY: f32 = 0.1;
    camera_sensitivity.x = camera_sensitivity.x.max(MIN_SENSITIVITY);
    camera_sensitivity.y = camera_sensitivity.y.max(MIN_SENSITIVITY);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn raise_camera_sensitivity(
    _trigger: Trigger<Pointer<Click>>,
    mut camera_sensitivity: ResMut<CameraSensitivity>,
) {
    camera_sensitivity.0 += 0.1;
    const MAX_SENSITIVITY: f32 = 20.0;
    camera_sensitivity.x = camera_sensitivity.x.min(MAX_SENSITIVITY);
    camera_sensitivity.y = camera_sensitivity.y.min(MAX_SENSITIVITY);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn update_camera_sensitivity_label(
    mut label: Single<&mut Text, With<CameraSensitivityLabel>>,
    camera_sensitivity: Res<CameraSensitivity>,
) {
    label.0 = format!("{:.1}", camera_sensitivity.x);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct CameraFovLabel;

fn lower_camera_fov(_trigger: Trigger<Pointer<Click>>, mut camera_fov: ResMut<WorldModelFov>) {
    camera_fov.0 -= 1.0;
    camera_fov.0 = camera_fov.0.max(45.0);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn raise_camera_fov(_trigger: Trigger<Pointer<Click>>, mut camera_fov: ResMut<WorldModelFov>) {
    camera_fov.0 += 1.0;
    camera_fov.0 = camera_fov.0.min(130.0);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn update_camera_fov_label(
    mut label: Single<&mut Text, With<CameraFovLabel>>,
    camera_fov: Res<WorldModelFov>,
) {
    label.0 = format!("{:.1}", camera_fov.0);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct GibCountLabel;

fn lower_gib_count(_trigger: Trigger<Pointer<Click>>, mut gore_settings: ResMut<GoreSettings>) {
    gore_settings.gib_count = gore_settings.gib_count.saturating_sub(1);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn raise_gib_count(_trigger: Trigger<Pointer<Click>>, mut gore_settings: ResMut<GoreSettings>) {
    gore_settings.gib_count += 1;
    gore_settings.gib_count = gore_settings.gib_count.min(11);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn update_gib_count_label(
    mut label: Single<&mut Text, With<GibCountLabel>>,
    gore_settings: Res<GoreSettings>,
) {
    label.0 = format!("{}", gore_settings.gib_count);
}

#[cfg_attr(feature = "hot_patch", hot)]
fn go_back_on_click(
    _trigger: Trigger<Pointer<Click>>,
    screen: Res<State<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}

#[cfg_attr(feature = "hot_patch", hot)]
fn go_back(screen: Res<State<Screen>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(if screen.get() == &Screen::Title {
        Menu::Main
    } else {
        Menu::Pause
    });
}
