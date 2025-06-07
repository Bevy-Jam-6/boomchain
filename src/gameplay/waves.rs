use std::time::Duration;

use avian3d::prelude::*;
use bevy::{prelude::*, time::Stopwatch};
use bevy_trenchbroom::prelude::*;
use rand::seq::SliceRandom as _;

use crate::{
    PrePhysicsAppSystems,
    gameplay::{
        hud::WaveIconParent,
        npc::{Npc, stats::NpcStats},
    },
    props::generic::BarrelLargeClosed,
    third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Waves>();
    app.register_type::<SpawnPackets>();
    app.register_type::<Spawner>();
    app.init_resource::<SpawnPackets>();
    app.add_systems(
        RunFixedMainLoop,
        advance_waves
            .in_set(PrePhysicsAppSystems::SpawnWave)
            .run_if(any_with_component::<WaveIconParent>),
    );
}

impl Default for Waves {
    fn default() -> Self {
        Self::new([
            Wave {
                prep_time: Millis(0),
                packet_kinds: [
                    (Millis(0), Difficulty(0)),
                    (Millis(5000), Difficulty(0)),
                    (Millis(10000), Difficulty(0)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(0)),
                    (Millis(5000), Difficulty(0)),
                    (Millis(10000), Difficulty(1)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(0)),
                    (Millis(2000), Difficulty(1)),
                    (Millis(6000), Difficulty(1)),
                    (Millis(11000), Difficulty(0)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(1)),
                    (Millis(4000), Difficulty(1)),
                    (Millis(7000), Difficulty(1)),
                    (Millis(11000), Difficulty(1)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(0)),
                    (Millis(0), Difficulty(0)),
                    (Millis(4000), Difficulty(1)),
                    (Millis(4000), Difficulty(1)),
                    (Millis(8000), Difficulty(1)),
                    (Millis(12000), Difficulty(1)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(2)),
                    (Millis(4000), Difficulty(1)),
                    (Millis(8000), Difficulty(1)),
                    (Millis(8000), Difficulty(1)),
                    (Millis(8000), Difficulty(0)),
                    (Millis(12000), Difficulty(1)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(2)),
                    (Millis(4000), Difficulty(2)),
                    (Millis(8000), Difficulty(2)),
                    (Millis(8000), Difficulty(1)),
                    (Millis(8000), Difficulty(0)),
                    (Millis(12000), Difficulty(1)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(2)),
                    (Millis(0), Difficulty(2)),
                    (Millis(8000), Difficulty(2)),
                    (Millis(8000), Difficulty(1)),
                    (Millis(8000), Difficulty(0)),
                    (Millis(12000), Difficulty(1)),
                    (Millis(12000), Difficulty(1)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(2)),
                    (Millis(0), Difficulty(1)),
                    (Millis(0), Difficulty(1)),
                    (Millis(3000), Difficulty(1)),
                    (Millis(3000), Difficulty(1)),
                    (Millis(8000), Difficulty(2)),
                    (Millis(8000), Difficulty(1)),
                    (Millis(8000), Difficulty(0)),
                    (Millis(8000), Difficulty(0)),
                    (Millis(12000), Difficulty(1)),
                    (Millis(12000), Difficulty(2)),
                ]
                .into(),
            },
            Wave {
                prep_time: Millis(10000),
                packet_kinds: [
                    (Millis(0), Difficulty(2)),
                    (Millis(0), Difficulty(2)),
                    (Millis(0), Difficulty(2)),
                    (Millis(0), Difficulty(1)),
                    (Millis(0), Difficulty(1)),
                    (Millis(0), Difficulty(1)),
                    (Millis(2000), Difficulty(2)),
                    (Millis(2000), Difficulty(2)),
                    (Millis(2000), Difficulty(2)),
                    (Millis(2000), Difficulty(2)),
                ]
                .into(),
            },
        ])
    }
}

impl Default for SpawnPackets {
    fn default() -> Self {
        Self(vec![
            SpawnPacket::new(
                Difficulty(0),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::BasicEnemy),
                    (Millis(300), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(0),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(300), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::ExplosiveBarrel),
                    (Millis(500), SpawnVariant::ExplosiveBarrel),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(0),
                [
                    (Millis(0), SpawnVariant::BigEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::BasicEnemy),
                    (Millis(300), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(0),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::ExplosiveBarrel),
                    (Millis(300), SpawnVariant::ExplosiveBarrel),
                    (Millis(400), SpawnVariant::ExplosiveBarrel),
                    (Millis(500), SpawnVariant::ExplosiveBarrel),
                    (Millis(600), SpawnVariant::ExplosiveBarrel),
                    (Millis(700), SpawnVariant::ExplosiveBarrel),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(1),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::BigEnemy),
                    (Millis(300), SpawnVariant::ExplosiveBarrel),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(500), SpawnVariant::ExplosiveBarrel),
                    (Millis(600), SpawnVariant::SmallEnemy),
                    (Millis(700), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(1),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BigEnemy),
                    (Millis(200), SpawnVariant::BigEnemy),
                    (Millis(300), SpawnVariant::BigEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BigEnemy),
                    (Millis(500), SpawnVariant::ExplosiveBarrel),
                    (Millis(600), SpawnVariant::SmallEnemy),
                    (Millis(700), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(1),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::BasicEnemy),
                    (Millis(300), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(500), SpawnVariant::BasicEnemy),
                    (Millis(600), SpawnVariant::BasicEnemy),
                    (Millis(700), SpawnVariant::BasicEnemy),
                    (Millis(800), SpawnVariant::ExplosiveBarrel),
                    (Millis(800), SpawnVariant::ExplosiveBarrel),
                    (Millis(900), SpawnVariant::BasicEnemy),
                    (Millis(1000), SpawnVariant::BasicEnemy),
                    (Millis(1100), SpawnVariant::BasicEnemy),
                    (Millis(1200), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(2),
                [
                    (Millis(0), SpawnVariant::SmallEnemy),
                    (Millis(100), SpawnVariant::SmallEnemy),
                    (Millis(200), SpawnVariant::SmallEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(300), SpawnVariant::SmallEnemy),
                    (Millis(400), SpawnVariant::SmallEnemy),
                    (Millis(500), SpawnVariant::BasicEnemy),
                    (Millis(600), SpawnVariant::BasicEnemy),
                    (Millis(700), SpawnVariant::BasicEnemy),
                    (Millis(800), SpawnVariant::ExplosiveBarrel),
                    (Millis(800), SpawnVariant::ExplosiveBarrel),
                    (Millis(900), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(2),
                [
                    (Millis(0), SpawnVariant::SmallEnemy),
                    (Millis(100), SpawnVariant::SmallEnemy),
                    (Millis(200), SpawnVariant::BigEnemy),
                    (Millis(300), SpawnVariant::BigEnemy),
                    (Millis(400), SpawnVariant::BigEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(500), SpawnVariant::BasicEnemy),
                    (Millis(600), SpawnVariant::BigEnemy),
                    (Millis(700), SpawnVariant::BasicEnemy),
                    (Millis(800), SpawnVariant::ExplosiveBarrel),
                    (Millis(800), SpawnVariant::ExplosiveBarrel),
                    (Millis(900), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(2),
                [
                    (Millis(0), SpawnVariant::BigEnemy),
                    (Millis(100), SpawnVariant::BigEnemy),
                    (Millis(200), SpawnVariant::BigEnemy),
                    (Millis(300), SpawnVariant::BigEnemy),
                    (Millis(400), SpawnVariant::BigEnemy),
                    (Millis(500), SpawnVariant::BigEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(600), SpawnVariant::BigEnemy),
                    (Millis(700), SpawnVariant::BigEnemy),
                    (Millis(800), SpawnVariant::BigEnemy),
                    (Millis(900), SpawnVariant::BigEnemy),
                    (Millis(1000), SpawnVariant::BigEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(2),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::BasicEnemy),
                    (Millis(300), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(500), SpawnVariant::BasicEnemy),
                    (Millis(600), SpawnVariant::BasicEnemy),
                    (Millis(700), SpawnVariant::BasicEnemy),
                    (Millis(800), SpawnVariant::BasicEnemy),
                    (Millis(900), SpawnVariant::BasicEnemy),
                    (Millis(1000), SpawnVariant::BasicEnemy),
                    (Millis(1100), SpawnVariant::BasicEnemy),
                    (Millis(1200), SpawnVariant::BasicEnemy),
                    (Millis(1300), SpawnVariant::BasicEnemy),
                    (Millis(1400), SpawnVariant::BasicEnemy),
                    (Millis(1500), SpawnVariant::BasicEnemy),
                    (Millis(1600), SpawnVariant::BasicEnemy),
                    (Millis(1700), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(3),
                [
                    (Millis(0), SpawnVariant::BasicEnemy),
                    (Millis(100), SpawnVariant::BasicEnemy),
                    (Millis(200), SpawnVariant::BasicEnemy),
                    (Millis(300), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(400), SpawnVariant::BasicEnemy),
                    (Millis(500), SpawnVariant::BasicEnemy),
                    (Millis(600), SpawnVariant::BasicEnemy),
                    (Millis(700), SpawnVariant::BasicEnemy),
                    (Millis(800), SpawnVariant::BasicEnemy),
                    (Millis(900), SpawnVariant::BasicEnemy),
                    (Millis(1000), SpawnVariant::BasicEnemy),
                    (Millis(1100), SpawnVariant::BasicEnemy),
                    (Millis(1200), SpawnVariant::BasicEnemy),
                    (Millis(1300), SpawnVariant::BasicEnemy),
                    (Millis(1400), SpawnVariant::BasicEnemy),
                    (Millis(1500), SpawnVariant::BasicEnemy),
                    (Millis(1600), SpawnVariant::BasicEnemy),
                    (Millis(1700), SpawnVariant::BasicEnemy),
                    (Millis(1800), SpawnVariant::BasicEnemy),
                    (Millis(1900), SpawnVariant::BasicEnemy),
                    (Millis(2000), SpawnVariant::BasicEnemy),
                    (Millis(2100), SpawnVariant::BasicEnemy),
                    (Millis(2200), SpawnVariant::BasicEnemy),
                    (Millis(2300), SpawnVariant::BasicEnemy),
                    (Millis(2400), SpawnVariant::BasicEnemy),
                    (Millis(2500), SpawnVariant::BasicEnemy),
                    (Millis(2600), SpawnVariant::BasicEnemy),
                    (Millis(2700), SpawnVariant::BasicEnemy),
                    (Millis(2800), SpawnVariant::BasicEnemy),
                    (Millis(2900), SpawnVariant::BasicEnemy),
                    (Millis(3000), SpawnVariant::BasicEnemy),
                    (Millis(3100), SpawnVariant::BasicEnemy),
                    (Millis(3200), SpawnVariant::BasicEnemy),
                    (Millis(3300), SpawnVariant::BasicEnemy),
                    (Millis(3400), SpawnVariant::BasicEnemy),
                    (Millis(3500), SpawnVariant::BasicEnemy),
                    (Millis(3600), SpawnVariant::BasicEnemy),
                    (Millis(3700), SpawnVariant::BasicEnemy),
                    (Millis(3800), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(3),
                [
                    (Millis(0), SpawnVariant::SmallEnemy),
                    (Millis(100), SpawnVariant::SmallEnemy),
                    (Millis(200), SpawnVariant::SmallEnemy),
                    (Millis(300), SpawnVariant::SmallEnemy),
                    (Millis(400), SpawnVariant::SmallEnemy),
                    (Millis(100), SpawnVariant::SmallEnemy),
                    (Millis(100), SpawnVariant::SmallEnemy),
                    (Millis(500), SpawnVariant::SmallEnemy),
                    (Millis(600), SpawnVariant::SmallEnemy),
                    (Millis(700), SpawnVariant::SmallEnemy),
                    (Millis(800), SpawnVariant::SmallEnemy),
                    (Millis(900), SpawnVariant::SmallEnemy),
                    (Millis(1000), SpawnVariant::SmallEnemy),
                    (Millis(1100), SpawnVariant::SmallEnemy),
                    (Millis(1200), SpawnVariant::SmallEnemy),
                    (Millis(1300), SpawnVariant::SmallEnemy),
                    (Millis(1400), SpawnVariant::SmallEnemy),
                    (Millis(1500), SpawnVariant::SmallEnemy),
                    (Millis(1600), SpawnVariant::SmallEnemy),
                ]
                .into(),
            ),
            SpawnPacket::new(
                Difficulty(3),
                [
                    (Millis(0), SpawnVariant::BigEnemy),
                    (Millis(100), SpawnVariant::BigEnemy),
                    (Millis(200), SpawnVariant::BigEnemy),
                    (Millis(300), SpawnVariant::BigEnemy),
                    (Millis(400), SpawnVariant::BigEnemy),
                    (Millis(500), SpawnVariant::BigEnemy),
                    (Millis(600), SpawnVariant::BigEnemy),
                    (Millis(700), SpawnVariant::BigEnemy),
                    (Millis(800), SpawnVariant::BigEnemy),
                    (Millis(900), SpawnVariant::ExplosiveBarrel),
                    (Millis(800), SpawnVariant::ExplosiveBarrel),
                    (Millis(900), SpawnVariant::ExplosiveBarrel),
                    (Millis(1000), SpawnVariant::ExplosiveBarrel),
                    (Millis(1100), SpawnVariant::ExplosiveBarrel),
                    (Millis(1200), SpawnVariant::ExplosiveBarrel),
                    (Millis(1300), SpawnVariant::ExplosiveBarrel),
                    (Millis(1400), SpawnVariant::SmallEnemy),
                    (Millis(1500), SpawnVariant::SmallEnemy),
                    (Millis(1600), SpawnVariant::SmallEnemy),
                    (Millis(1700), SpawnVariant::BasicEnemy),
                    (Millis(1800), SpawnVariant::BasicEnemy),
                    (Millis(1900), SpawnVariant::BasicEnemy),
                    (Millis(2000), SpawnVariant::BasicEnemy),
                    (Millis(2100), SpawnVariant::BasicEnemy),
                    (Millis(2200), SpawnVariant::BasicEnemy),
                    (Millis(2300), SpawnVariant::BasicEnemy),
                    (Millis(2400), SpawnVariant::BasicEnemy),
                    (Millis(2500), SpawnVariant::BasicEnemy),
                    (Millis(2600), SpawnVariant::BasicEnemy),
                    (Millis(2700), SpawnVariant::BasicEnemy),
                    (Millis(2800), SpawnVariant::BasicEnemy),
                    (Millis(2900), SpawnVariant::BasicEnemy),
                    (Millis(3000), SpawnVariant::BasicEnemy),
                ]
                .into(),
            ),
        ])
    }
}

#[derive(Event)]
pub(crate) struct WaveAdvanced;

#[derive(Event)]
pub(crate) struct WaveWaitingForEnemies;

#[derive(Event)]
pub(crate) struct WaveStartedPreparing;

#[derive(Event)]
pub(crate) struct WaveFinishedPreparing;

#[derive(Event)]
pub(crate) struct GameWon;

fn advance_waves(
    mut waves: Single<&mut Waves>,
    packets: Res<SpawnPackets>,
    time: Res<Time>,
    enemies: Query<(), With<Npc>>,
    spawners: Query<(&Transform, &Spawner)>,
    spatial_query: SpatialQuery,
    mut commands: Commands,
) {
    let is_preparing_before = waves.is_preparing();
    let advancement = waves.try_advance(time.delta(), !enemies.is_empty());
    let is_preparing_after = waves.is_preparing();

    match advancement {
        WaveAdvancement::Advanced => {
            commands.trigger(WaveAdvanced);
        }
        WaveAdvancement::WaitingForEnemies => {
            commands.trigger(WaveWaitingForEnemies);
        }
        WaveAdvancement::Ongoing => {}
    }

    if !is_preparing_before && is_preparing_after {
        commands.trigger(WaveStartedPreparing);
    }

    if is_preparing_before && !is_preparing_after {
        commands.trigger(WaveFinishedPreparing);
    }

    if waves.is_finished() {
        if enemies.is_empty() {
            commands.trigger(GameWon);
        } else {
            info_once!("Game finished, but there are still enemies");
        }
        return;
    }
    if !is_preparing_after {
        let difficulties = waves.pop_difficulties_to_spawn();
        for difficulty in difficulties {
            let available_packets = packets.filter_difficulty(difficulty);
            let Some(packet) = available_packets.choose(&mut rand::thread_rng()) else {
                error!("No packets available for difficulty {difficulty}");
                continue;
            };
            waves.current_packets.push(packet.clone());
        }
        let spawns = waves
            .current_packets
            .iter_mut()
            .flat_map(|packet| packet.pop_spawns())
            .collect::<Vec<_>>();

        waves.clean_finished_packets();
        let spawners = spawners.iter().collect::<Vec<_>>();
        for spawn in spawns {
            let Some((transform, spawner)) = spawners.choose(&mut rand::thread_rng()) else {
                error!("No spawners available");
                continue;
            };
            let spawner_transform = transform.translation;
            let spawner_radius = spawner.radius;
            let pos2 = Circle::new(spawner_radius).sample_interior(&mut rand::thread_rng());
            let pos3 = Vec3::new(pos2.x, 0.0, pos2.y);
            let try_spawn_position = spawner_transform + pos3;
            let Ok(dir) = Dir3::try_from(try_spawn_position - spawner_transform) else {
                error!("Invalid direction, skipping spawn");
                continue;
            };
            let filter = SpatialQueryFilter::default().with_mask([CollisionLayer::Default]);
            let spawn_position = if let Some(hit) =
                spatial_query.cast_ray(spawner_transform, dir, pos3.length(), true, &filter)
            {
                spawner_transform + dir * (hit.distance - 1.0).max(0.0)
            } else {
                try_spawn_position
            };
            let mut spawn_commands = commands.spawn((
                Visibility::Inherited,
                Transform::from_translation(spawn_position),
            ));
            match spawn {
                SpawnVariant::BasicEnemy => {
                    spawn_commands.insert((
                        Name::new("Basic Enemy"),
                        Npc,
                        NpcStats {
                            health: 100.0,
                            desired_speed: 7.0,
                            max_speed: 8.0,
                            attack_damage: 10.0,
                            attack_speed_range: 1.5..2.3,
                            size: 1.0,
                            stagger_chance: 0.3,
                            stagger_duration: 0.2..0.4,
                        },
                    ));
                }
                SpawnVariant::BigEnemy => {
                    spawn_commands.insert((
                        Name::new("Big Enemy"),
                        Npc,
                        NpcStats {
                            health: 400.0,
                            desired_speed: 5.0,
                            max_speed: 5.0,
                            attack_damage: 40.0,
                            attack_speed_range: 1.1..1.7,
                            size: 2.0,
                            stagger_chance: 0.2,
                            stagger_duration: 0.1..0.3,
                        },
                    ));
                }
                SpawnVariant::SmallEnemy => {
                    spawn_commands.insert((
                        Name::new("Small Enemy"),
                        Npc,
                        NpcStats {
                            health: 30.0,
                            desired_speed: 11.0,
                            max_speed: 11.0,
                            attack_damage: 10.0,
                            attack_speed_range: 2.1..2.8,
                            size: 0.7,
                            stagger_chance: 0.5,
                            stagger_duration: 0.2..0.3,
                        },
                    ));
                }
                SpawnVariant::ExplosiveBarrel => {
                    spawn_commands.insert((Name::new("Explosive Barrel"), BarrelLargeClosed));
                }
            }
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct Waves {
    waves: Vec<Wave>,
    current_packets: Vec<SpawnPacket>,
    wave_stopwatch: Stopwatch,
    current_wave: usize,
    total_waves: usize,
    prep_timer: Timer,
}

enum WaveAdvancement {
    Advanced,
    WaitingForEnemies,
    Ongoing,
}

impl Waves {
    fn new(waves: impl Into<Vec<Wave>>) -> Self {
        let waves = waves.into();
        let len = waves.len();
        Self {
            waves,
            current_packets: Vec::new(),
            wave_stopwatch: Stopwatch::default(),
            current_wave: 0,
            total_waves: len,
            prep_timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }

    pub(crate) fn current_wave_index(&self) -> usize {
        self.current_wave
    }

    pub(crate) fn total_waves(&self) -> usize {
        self.total_waves
    }

    pub(crate) fn prep_time_left(&self) -> Duration {
        self.prep_timer.remaining()
    }

    fn try_advance(&mut self, delta: Duration, has_enemies: bool) -> WaveAdvancement {
        let mut advancement = WaveAdvancement::Ongoing;
        if !self.is_finished()
            && self
                .current_wave()
                .map(|wave| wave.packet_kinds.is_empty())
                .unwrap_or(false)
        {
            if has_enemies {
                advancement = WaveAdvancement::WaitingForEnemies;
            } else {
                self.advance_wave();
                advancement = WaveAdvancement::Advanced;
            }
        }
        if self.is_preparing() {
            self.prep_timer.tick(delta);
        } else {
            self.wave_stopwatch.tick(delta);
            for packet in self.current_packets.iter_mut() {
                packet.tick(delta);
            }
        }
        advancement
    }

    fn clean_finished_packets(&mut self) {
        self.current_packets
            .retain(|packet| !packet.spawns.is_empty());
    }

    fn current_wave(&self) -> Option<&Wave> {
        self.waves.get(self.current_wave)
    }

    fn current_wave_mut(&mut self) -> Option<&mut Wave> {
        self.waves.get_mut(self.current_wave)
    }

    fn elapsed_millis(&self) -> Millis {
        self.wave_stopwatch.elapsed().into()
    }

    fn pop_difficulties_to_spawn(&mut self) -> Vec<Difficulty> {
        let mut difficulties = Vec::new();

        let elapsed = self.elapsed_millis();
        let Some(current_wave) = self.current_wave() else {
            return difficulties;
        };
        for (millis, difficulty) in current_wave.packet_kinds_ordered() {
            if elapsed > millis {
                difficulties.push(difficulty);
                self.current_wave_mut()
                    .unwrap()
                    .packet_kinds
                    .retain(|(m, _)| *m != millis);
            }
        }
        difficulties
    }

    fn is_preparing(&self) -> bool {
        !self.prep_timer.finished()
    }
    fn advance_wave(&mut self) {
        self.current_wave += 1;
        let prep_time = if let Some(current_wave) = self.current_wave() {
            current_wave.prep_time
        } else {
            Millis(0)
        };
        self.prep_timer = Timer::new(Duration::from_millis(prep_time.0), TimerMode::Once);
        self.wave_stopwatch.reset();
    }

    fn is_finished(&self) -> bool {
        self.current_wave >= self.waves.len() && self.current_packets.is_empty()
    }
}

#[derive(PointClass, Component, Debug, Reflect)]
#[reflect(QuakeClass, Component)]
#[base(Transform, Visibility)]
#[model("models/gizmo/spawner.gltf")]
struct Spawner {
    radius: f32,
}

impl Default for Spawner {
    fn default() -> Self {
        Self { radius: 5.0 }
    }
}

#[derive(Reflect, Debug)]
struct Wave {
    prep_time: Millis,
    packet_kinds: Vec<(Millis, Difficulty)>,
}

impl Wave {
    fn packet_kinds_ordered(&self) -> Vec<(Millis, Difficulty)> {
        let mut vector = self
            .packet_kinds
            .iter()
            .map(|(millis, difficulty)| (*millis, *difficulty))
            .collect::<Vec<_>>();
        vector.sort_by_key(|(millis, _)| *millis);
        vector
    }
}

#[derive(Deref, DerefMut, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect, Copy, Clone, Debug)]
struct Millis(u64);

impl std::fmt::Display for Millis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ms", self.0)
    }
}

impl From<Duration> for Millis {
    fn from(duration: Duration) -> Self {
        let millis = duration.as_millis();
        if millis > u64::MAX as u128 {
            error!("Duration too long to convert to Millis");
            Millis(u64::MAX)
        } else {
            Self(millis as u64)
        }
    }
}

#[derive(Deref, DerefMut, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect, Copy, Clone, Debug)]
struct Difficulty(u32);

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct SpawnPackets(Vec<SpawnPacket>);

impl SpawnPackets {
    fn filter_difficulty(&self, difficulty: Difficulty) -> Vec<SpawnPacket> {
        self.0
            .iter()
            .filter(|packet| packet.difficulty == difficulty)
            .cloned()
            .collect()
    }
}

#[derive(Reflect, Clone)]
struct SpawnPacket {
    difficulty: Difficulty,
    stopwatch: Stopwatch,
    spawns: Vec<(Millis, SpawnVariant)>,
}

impl SpawnPacket {
    fn new(difficulty: Difficulty, spawns: Vec<(Millis, SpawnVariant)>) -> Self {
        Self {
            difficulty,
            stopwatch: Stopwatch::default(),
            spawns,
        }
    }

    fn pop_spawns(&mut self) -> Vec<SpawnVariant> {
        let mut spawns = Vec::new();
        for (millis, spawn_variant) in self
            .spawns
            .iter()
            .map(|(millis, spawn_variant)| (*millis, *spawn_variant))
            .collect::<Vec<_>>()
        {
            if self.elapsed_millis() > millis {
                spawns.push(spawn_variant);
                self.spawns.retain(|(m, _)| *m != millis);
            }
        }
        spawns
    }

    fn tick(&mut self, delta: Duration) {
        self.stopwatch.tick(delta);
    }

    fn elapsed_millis(&self) -> Millis {
        self.stopwatch.elapsed().into()
    }
}

#[derive(Reflect, Clone, Copy)]
enum SpawnVariant {
    BasicEnemy,
    BigEnemy,
    SmallEnemy,
    ExplosiveBarrel,
}
