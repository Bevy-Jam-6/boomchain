use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*, time::Stopwatch};
use bevy_trenchbroom::prelude::*;
use rand::seq::SliceRandom as _;

use crate::PrePhysicsAppSystems;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Waves>();
    app.register_type::<SpawnPackets>();
    app.register_type::<Spawner>();
    app.init_resource::<SpawnPackets>();
    app.add_systems(
        RunFixedMainLoop,
        advance_waves.in_set(PrePhysicsAppSystems::SpawnWave),
    );
}

impl Default for Waves {
    fn default() -> Self {
        Self::new([Wave {
            prep_time: Millis(0),
            packet_kinds: [(Millis(0), Difficulty(0))].into(),
        }])
    }
}

impl Default for SpawnPackets {
    fn default() -> Self {
        Self(vec![SpawnPacket {
            difficulty: Difficulty(0),
            spawns: [(Millis(0), SpawnVariant::BasicEnemy)].into(),
        }])
    }
}

fn advance_waves(mut waves: Single<&mut Waves>, packets: Res<SpawnPackets>, time: Res<Time>) {
    info!("tick");
    waves.tick(time.delta());
    if waves.is_finished() {
        info!("Game finished");
        return;
    }
    if waves.is_preparing() {
        info!("Preparing wave {}", waves.current_wave);
        info!("Prep time left: {}", waves.current_wave().prep_time);
    } else {
        let difficulties = waves.pop_difficulties_to_spawn();
        for difficulty in difficulties {
            let available_packets = packets.filter_difficulty(difficulty);
            let Some(packet) = available_packets.choose(&mut rand::thread_rng()) else {
                error!("No packets available for difficulty {difficulty}");
                continue;
            };
            for (millis, spawn_variant) in packet.spawns.iter() {
                if waves.elapsed_millis() > *millis {
                    match spawn_variant {
                        SpawnVariant::BasicEnemy => {
                            info!("Spawning BasicEnemy at {}", waves.elapsed_millis());
                        }
                        SpawnVariant::ExplosiveBarrel => {
                            info!("Spawning ExplosiveBarrel at {}", waves.elapsed_millis());
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct Waves {
    waves: Vec<Wave>,
    wave_stopwatch: Stopwatch,
    current_wave: usize,
    prep_timer: Timer,
}

impl Waves {
    fn new(waves: impl Into<Vec<Wave>>) -> Self {
        Self {
            waves: waves.into(),
            wave_stopwatch: Stopwatch::default(),
            current_wave: 0,
            prep_timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }

    fn tick(&mut self, delta: Duration) {
        if !self.is_finished() && self.current_wave().packet_kinds.is_empty() {
            self.advance_wave();
        }
        if self.is_preparing() {
            self.prep_timer.tick(delta);
        } else {
            self.wave_stopwatch.tick(delta);
        }
    }

    fn current_wave(&self) -> &Wave {
        &self.waves[self.current_wave]
    }

    fn elapsed_millis(&self) -> Millis {
        self.wave_stopwatch.elapsed().into()
    }

    fn pop_difficulties_to_spawn(&mut self) -> Vec<Difficulty> {
        let mut difficulties = Vec::new();

        let elapsed = self.elapsed_millis();
        for (millis, difficulty) in self.current_wave().packet_kinds_ordered() {
            if elapsed > millis {
                difficulties.push(difficulty);
                self.current_wave_mut().packet_kinds.remove(&millis);
            } else {
                break;
            }
        }
        difficulties
    }

    fn is_preparing(&self) -> bool {
        !self.prep_timer.finished()
    }

    fn current_wave_mut(&mut self) -> &mut Wave {
        &mut self.waves[self.current_wave]
    }

    fn advance_wave(&mut self) {
        self.current_wave += 1;
        let prep_time = self.current_wave().prep_time;
        self.prep_timer = Timer::new(Duration::from_millis(prep_time.0), TimerMode::Once);
        self.wave_stopwatch.reset();
    }

    fn is_finished(&self) -> bool {
        self.current_wave >= self.waves.len()
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

#[derive(Reflect)]
struct Wave {
    prep_time: Millis,
    packet_kinds: HashMap<Millis, Difficulty>,
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

#[derive(Deref, DerefMut, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect, Copy, Clone)]
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

#[derive(Deref, DerefMut, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect, Copy, Clone)]
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
    spawns: HashMap<Millis, SpawnVariant>,
}

#[derive(Reflect, Clone, Copy)]
enum SpawnVariant {
    BasicEnemy,
    ExplosiveBarrel,
}
