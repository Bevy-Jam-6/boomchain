use std::time::Duration;

use bevy::{platform::collections::HashMap, prelude::*, time::Stopwatch};
use bevy_trenchbroom::prelude::*;
use rand::seq::SliceRandom as _;

use crate::{PrePhysicsAppSystems, gameplay::npc::Npc};

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
            packet_kinds: [(Millis(0), Difficulty(0)), (Millis(5000), Difficulty(0))].into(),
        }])
    }
}

impl Default for SpawnPackets {
    fn default() -> Self {
        Self(vec![SpawnPacket::new(
            Difficulty(0),
            [
                (Millis(0), SpawnVariant::BasicEnemy),
                (Millis(100), SpawnVariant::BasicEnemy),
                (Millis(200), SpawnVariant::BasicEnemy),
                (Millis(300), SpawnVariant::BasicEnemy),
                (Millis(400), SpawnVariant::BasicEnemy),
                (Millis(500), SpawnVariant::BasicEnemy),
                (Millis(600), SpawnVariant::BasicEnemy),
            ]
            .into(),
        )])
    }
}

fn advance_waves(
    mut waves: Single<&mut Waves>,
    packets: Res<SpawnPackets>,
    time: Res<Time>,
    enemies: Query<(), With<Npc>>,
    spawners: Query<(&Transform, &Spawner)>,
    mut commands: Commands,
) {
    waves.tick(time.delta());
    if waves.is_finished() {
        if enemies.is_empty() {
            info_once!("Game finished");
        } else {
            info_once!("Game finished, but there are still enemies");
        }
        return;
    }
    if waves.is_preparing() {
        info!("Preparing wave {}", waves.current_wave);
        info!(
            "Prep time left: {}",
            waves
                .current_wave()
                .expect("Is preparing, but there is not current wave")
                .prep_time
        );
    } else {
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
            match spawn {
                SpawnVariant::BasicEnemy => {
                    let Some((transform, spawner)) = spawners.choose(&mut rand::thread_rng())
                    else {
                        error!("No spawners available");
                        continue;
                    };
                    info!("Spawning BasicEnemy at {}", waves.elapsed_millis());
                    let spawner_transform = transform.translation;
                    let spawner_radius = spawner.radius;
                    let pos2 = Circle::new(spawner_radius).sample_interior(&mut rand::thread_rng());
                    let pos3 = Vec3::new(pos2.x, 0.0, pos2.y);
                    let spawn_position = spawner_transform + pos3;
                    commands.spawn((
                        Npc,
                        Visibility::Inherited,
                        Transform::from_translation(spawn_position),
                    ));
                }
                SpawnVariant::ExplosiveBarrel => {
                    info!("Spawning ExplosiveBarrel at {}", waves.elapsed_millis());
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
    prep_timer: Timer,
}

impl Waves {
    fn new(waves: impl Into<Vec<Wave>>) -> Self {
        Self {
            waves: waves.into(),
            current_packets: Vec::new(),
            wave_stopwatch: Stopwatch::default(),
            current_wave: 0,
            prep_timer: Timer::from_seconds(0.0, TimerMode::Once),
        }
    }

    fn tick(&mut self, delta: Duration) {
        if !self.is_finished()
            && self
                .current_wave()
                .map(|wave| wave.packet_kinds.is_empty())
                .unwrap_or(false)
        {
            self.advance_wave();
        }
        if self.is_preparing() {
            self.prep_timer.tick(delta);
        } else {
            self.wave_stopwatch.tick(delta);
            for packet in self.current_packets.iter_mut() {
                packet.tick(delta);
            }
        }
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
                    .remove(&millis);
            } else {
                break;
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
    spawns: HashMap<Millis, SpawnVariant>,
}

impl SpawnPacket {
    fn new(difficulty: Difficulty, spawns: HashMap<Millis, SpawnVariant>) -> Self {
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
                self.spawns.remove(&millis);
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
    ExplosiveBarrel,
}
