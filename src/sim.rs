use bevy::color::LinearRgba;
use bevy::prelude::*;
use rand::{Rng, RngCore, SeedableRng};
use std::collections::{HashMap, HashSet};

use crate::quadtree::{Quad, QuadTree};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Playing,
    GameOver,
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum SimState {
    #[default]
    Parallel,
    Sequential,
}

#[derive(Resource)]
pub struct SeededRng(pub rand::rngs::StdRng);

#[derive(Resource)]
struct TrailSpawnTimer(Timer);

#[derive(Resource)]
struct HazardSpawnTimer(Timer);

#[derive(Event)]
struct BodyAbsorbed {
    winner: Entity,
    loser_mass: f32,
    loser_vel: Vec2,
    loser_class: Class,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Objective {
    #[default]
    None,
    Survive,
}

#[derive(Resource)]
pub struct Mission {
    pub objective: Objective,
    pub progress: f32,
    pub goal: f32,
    pub completed: bool,
}

impl Default for Mission {
    fn default() -> Self {
        Self {
            objective: Objective::Survive,
            progress: 0.0,
            goal: 60.0, // Survive for 60 seconds
            completed: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SystemType {
    #[default]
    SingleStar,
    BinaryStar,
    Cluster,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Scenario {
    #[default]
    CalmBelts,
    BinaryMayhem,
    StarNursery,
    BHArena,
}

pub struct SimPlugin;
impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimSettings>()
            .init_resource::<SimStats>()
            .init_resource::<Mission>()
            .insert_resource(TrailSpawnTimer(Timer::from_seconds(
                0.05,
                TimerMode::Repeating,
            )))
            .insert_resource(HazardSpawnTimer(Timer::from_seconds(
                15.0,
                TimerMode::Repeating,
            )))
            .add_event::<SpawnBurst>()
            .add_event::<PlayerDied>()
            .add_event::<ResetEvent>()
            .add_event::<BodyAbsorbed>()
            .add_systems(Startup, (spawn_initial_bodies, spawn_player))
            .add_systems(Update, (handle_reset, update_mission, player_death_system))
            .add_systems(OnEnter(SimState::Sequential), |mut commands: Commands| {
                commands.insert_resource(SeededRng(rand::SeedableRng::from_seed([0; 32])));
            })
            .add_systems(OnExit(SimState::Sequential), |mut commands: Commands| {
                commands.remove_resource::<SeededRng>();
            })
            .add_systems(
                Update,
                (
                    kick1_drift,
                    rebuild_quadtree,
                    apply_bh_forces,
                    kick2,
                    spatial_hash_build,
                    resolve_collisions,
                    update_render,
                    spawn_bursts,
                    spawn_trails,
                    update_trails,
                    check_player_evolution,
                    update_score,
                    spawn_hazards,
                )
                    .run_if(in_state(SimState::Parallel))
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (
                    kick1_drift,
                    rebuild_quadtree,
                    apply_bh_forces,
                    kick2,
                    spatial_hash_build,
                    resolve_collisions,
                    update_render,
                    spawn_bursts,
                    spawn_trails,
                    update_trails,
                    check_player_evolution,
                    update_score,
                    spawn_hazards,
                )
                    .chain()
                    .run_if(in_state(SimState::Sequential))
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Asteroid,
    Planet,
    Star,
    BlackHole,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CollisionMode {
    #[default]
    Absorb,
    Elastic,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ColorPalette {
    #[default]
    Default,
    Colorblind,
}

impl Class {
    pub fn from_mass(m: f32) -> Self {
        if m < 500.0 {
            Class::Asteroid
        } else if m < 20_000.0 {
            Class::Planet
        } else if m < 1_000_000.0 {
            Class::Star
        } else {
            Class::BlackHole
        }
    }
    pub fn radius_for_mass(m: f32) -> f32 {
        match Class::from_mass(m) {
            Class::Asteroid => (m.sqrt() * 0.12).clamp(1.2, 6.0),
            Class::Planet => (m.sqrt() * 0.07).clamp(6.0, 16.0),
            Class::Star => (m.powf(0.33) * 0.6).clamp(16.0, 32.0),
            Class::BlackHole => (m.powf(0.25) * 0.9).clamp(32.0, 60.0),
        }
    }
    pub fn color(&self, palette: ColorPalette) -> Color {
        match palette {
            ColorPalette::Default => match *self {
                Class::Asteroid => Color::srgb(0.75, 0.8, 1.0),
                Class::Planet => Color::srgb(0.6, 0.9, 1.0),
                Class::Star => Color::srgb(1.0, 0.92, 0.6),
                Class::BlackHole => Color::BLACK,
            },
            ColorPalette::Colorblind => match *self {
                // Using Okabe-Ito palette
                Class::Asteroid => Color::srgb(0.6, 0.6, 0.6), // Gray
                Class::Planet => Color::srgb(0.0, 0.447, 0.698), // Blue
                Class::Star => Color::srgb(0.902, 0.624, 0.0), // Orange
                Class::BlackHole => Color::BLACK,
            },
        }
    }
    pub fn glow(&self) -> f32 {
        match *self {
            Class::Asteroid => 1.0, // No glow
            Class::Planet => 1.2,
            Class::Star => 3.0,
            Class::BlackHole => 0.0, // Black holes don't emit light
        }
    }
    pub fn rarity(&self) -> f32 {
        match *self {
            Class::Asteroid => 1.0,
            Class::Planet => 5.0,
            Class::Star => 25.0,
            Class::BlackHole => 100.0,
        }
    }
}

#[derive(Event)]
pub struct SpawnBurst {
    pub center: Vec2,
    pub radius: f32,
    pub count: usize,
    pub base_mass: f32,
    pub speed: f32,
}

#[derive(Event, Default)]
pub struct PlayerDied;

#[derive(Event, Default)]
pub struct ResetEvent;

#[derive(Resource, Clone)]
pub struct SimSettings {
    pub g: f32,
    pub dt: f32,
    pub softening: f32,
    pub max_vel: f32,
    pub theta: f32,
    pub running: bool,
    pub trails_enabled: bool,
    pub trail_lifespan: f32,
    pub spawn_limit: usize,
    pub restitution: f32,
    pub absorb_bias: f32,
    pub collision_mode: CollisionMode,
    pub deterministic: bool,
    pub follow_player: bool,
    pub time_scale: f32,
    pub show_help: bool,
    pub show_diagnostics: bool,
    pub color_palette: ColorPalette,
    pub system_type: SystemType,
    pub scenario: Scenario,
    // New adaptive settings
    pub adaptive_theta: bool,
    pub theta_range: Vec2, // min, max
    pub adaptive_softening: bool,
    pub softening_range: Vec2, // min, max
}
impl Default for SimSettings {
    fn default() -> Self {
        Self {
            g: 120.0,
            dt: 0.008,
            softening: 4.0,
            max_vel: 1800.0,
            theta: 0.6,
            running: true,
            trails_enabled: true,
            trail_lifespan: 1.5,
            spawn_limit: 50_000,
            restitution: 0.8,
            absorb_bias: 0.03,
            collision_mode: CollisionMode::default(),
            deterministic: false,
            follow_player: true,
            time_scale: 1.0,
            show_help: true,
            show_diagnostics: false,
            color_palette: ColorPalette::default(),
            system_type: SystemType::default(),
            scenario: Scenario::default(),
            adaptive_theta: true,
            theta_range: Vec2::new(0.4, 1.0),
            adaptive_softening: true,
            softening_range: Vec2::new(2.0, 10.0),
        }
    }
}

impl SimSettings {
    pub fn from_scenario(scenario: Scenario) -> Self {
        let mut settings = SimSettings::default();
        settings.scenario = scenario;
        match scenario {
            Scenario::CalmBelts => {
                settings.g = 120.0;
                settings.dt = 0.008;
                settings.softening = 4.0;
                settings.max_vel = 1800.0;
                settings.theta = 0.6;
                settings.system_type = SystemType::SingleStar;
                settings.collision_mode = CollisionMode::Absorb;
                settings.restitution = 0.0;
                settings.absorb_bias = 0.03;
                settings.trails_enabled = true;
                settings.trail_lifespan = 1.5;
                settings.deterministic = false;
                settings.follow_player = true;
                settings.time_scale = 1.0;
                settings.show_help = true;
                settings.show_diagnostics = false;
                settings.color_palette = ColorPalette::Default;
                settings.adaptive_theta = true;
                settings.theta_range = Vec2::new(0.4, 1.0);
                settings.adaptive_softening = true;
                settings.softening_range = Vec2::new(2.0, 10.0);
            }
            Scenario::BinaryMayhem => {
                settings.g = 200.0;
                settings.dt = 0.005;
                settings.softening = 8.0;
                settings.max_vel = 2500.0;
                settings.theta = 0.8;
                settings.system_type = SystemType::BinaryStar;
                settings.collision_mode = CollisionMode::Elastic;
                settings.restitution = 0.9;
                settings.absorb_bias = 0.0;
                settings.trails_enabled = true;
                settings.trail_lifespan = 2.0;
                settings.deterministic = false;
                settings.follow_player = true;
                settings.time_scale = 1.0;
                settings.show_help = true;
                settings.show_diagnostics = false;
                settings.color_palette = ColorPalette::Default;
                settings.adaptive_theta = true;
                settings.theta_range = Vec2::new(0.6, 1.2);
                settings.adaptive_softening = true;
                settings.softening_range = Vec2::new(5.0, 15.0);
            }
            Scenario::StarNursery => {
                settings.g = 150.0;
                settings.dt = 0.01;
                settings.softening = 6.0;
                settings.max_vel = 2000.0;
                settings.theta = 0.7;
                settings.system_type = SystemType::Cluster;
                settings.collision_mode = CollisionMode::Absorb;
                settings.restitution = 0.0;
                settings.absorb_bias = 0.05;
                settings.trails_enabled = true;
                settings.trail_lifespan = 1.8;
                settings.deterministic = false;
                settings.follow_player = true;
                settings.time_scale = 1.0;
                settings.show_help = true;
                settings.show_diagnostics = false;
                settings.color_palette = ColorPalette::Default;
                settings.adaptive_theta = true;
                settings.theta_range = Vec2::new(0.5, 1.1);
                settings.adaptive_softening = true;
                settings.softening_range = Vec2::new(3.0, 12.0);
            }
            Scenario::BHArena => {
                settings.g = 300.0;
                settings.dt = 0.003;
                settings.softening = 10.0;
                settings.max_vel = 3000.0;
                settings.theta = 0.9;
                settings.system_type = SystemType::SingleStar; // Will spawn a BH as central star
                settings.collision_mode = CollisionMode::Absorb;
                settings.restitution = 0.0;
                settings.absorb_bias = 0.1;
                settings.trails_enabled = true;
                settings.trail_lifespan = 2.5;
                settings.deterministic = false;
                settings.follow_player = true;
                settings.time_scale = 1.0;
                settings.show_help = true;
                settings.show_diagnostics = false;
                settings.color_palette = ColorPalette::Default;
                settings.adaptive_theta = true;
                settings.theta_range = Vec2::new(0.7, 1.5);
                settings.adaptive_softening = true;
                settings.softening_range = Vec2::new(8.0, 20.0);
            }
        }
        settings
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SimStats(pub usize);

#[derive(Component)]
pub struct Body {
    pub mass: f32,
    pub vel: Vec2,
    pub acc: Vec2,
    pub class: Class,
}

#[derive(Component)]
pub struct Player {
    pub prev_class: Class,
    pub score: f32,
}

#[derive(Component)]
pub struct Trail {
    pub lifespan: f32,
}

#[derive(Component)]
struct SmoothSize {
    target_radius: f32,
}

#[derive(Component)]
pub struct Hazard;

#[derive(Resource)]
struct TreeState {
    root: Option<QuadTree>,
    bounds: Quad,
}
impl Default for TreeState {
    fn default() -> Self {
        Self {
            root: None,
            bounds: Quad::new(Vec2::ZERO, 10000.0),
        }
    }
}

fn spawn_initial_bodies_inner(
    commands: &mut Commands,
    stats: &mut SimStats,
    settings: &SimSettings,
) {
    commands.insert_resource(TreeState::default());

    let mut rng: Box<dyn RngCore> = if settings.deterministic {
        Box::new(rand::rngs::StdRng::from_seed([0; 32]))
    } else {
        Box::new(rand::thread_rng())
    };

    match settings.system_type {
        SystemType::SingleStar => {
            // Central star
            let m = 6e5;
            let class = Class::from_mass(m);
            commands.spawn((
                Body {
                    mass: m,
                    vel: Vec2::ZERO,
                    acc: Vec2::ZERO,
                    class,
                },
                SmoothSize {
                    target_radius: Class::radius_for_mass(m),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: class.color(settings.color_palette),
                        custom_size: Some(Vec2::splat(Class::radius_for_mass(m))),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::ZERO),
                    ..default()
                },
            ));

            // Belts
            for r in [260.0, 520.0, 980.0, 1600.0] {
                for _ in 0..500 {
                    let ang = rng.gen::<f32>() * std::f32::consts::TAU;
                    let pos = Vec2::from_angle(ang) * (r + rng.gen::<f32>() * 40.0 - 20.0);
                    let vdir = Vec2::new(-pos.y, pos.x).normalize();
                    let v = vdir * (pos.length().sqrt() * 3.2);
                    let mass = rng.gen_range(6.0..60.0);
                    let class = Class::from_mass(mass);
                    commands.spawn((
                        Body {
                            mass,
                            vel: v,
                            acc: Vec2::ZERO,
                            class,
                        },
                        SmoothSize {
                            target_radius: Class::radius_for_mass(mass),
                        },
                        SpriteBundle {
                            sprite: Sprite {
                                color: class.color(settings.color_palette),
                                custom_size: Some(Vec2::splat(Class::radius_for_mass(mass))),
                                ..default()
                            },
                            transform: Transform::from_translation(pos.extend(0.0)),
                            ..default()
                        },
                    ));
                    stats.0 += 1;
                }
            }
        }
        SystemType::BinaryStar => {
            let m1 = 4e5;
            let m2 = 2e5;
            let class1 = Class::from_mass(m1);
            let class2 = Class::from_mass(m2);
            let r = 300.0;

            let v1 = (settings.g * m2 / (r * 2.0)).sqrt();
            let v2 = (settings.g * m1 / (r * 2.0)).sqrt();

            commands.spawn((
                Body {
                    mass: m1,
                    vel: Vec2::new(0.0, v1),
                    acc: Vec2::ZERO,
                    class: class1,
                },
                SmoothSize {
                    target_radius: Class::radius_for_mass(m1),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: class1.color(settings.color_palette),
                        custom_size: Some(Vec2::splat(Class::radius_for_mass(m1))),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(-r, 0.0, 0.0)),
                    ..default()
                },
            ));

            commands.spawn((
                Body {
                    mass: m2,
                    vel: Vec2::new(0.0, -v2),
                    acc: Vec2::ZERO,
                    class: class2,
                },
                SmoothSize {
                    target_radius: Class::radius_for_mass(m2),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: class2.color(settings.color_palette),
                        custom_size: Some(Vec2::splat(Class::radius_for_mass(m2))),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(r, 0.0, 0.0)),
                    ..default()
                },
            ));
        }
        SystemType::Cluster => {
            for _ in 0..50 {
                let pos = Vec2::new(
                    rng.gen_range(-1000.0..1000.0),
                    rng.gen_range(-1000.0..1000.0),
                );
                let mass = rng.gen_range(1000.0..50000.0);
                let class = Class::from_mass(mass);
                commands.spawn((
                    Body {
                        mass,
                        vel: Vec2::ZERO,
                        acc: Vec2::ZERO,
                        class,
                    },
                    SmoothSize {
                        target_radius: Class::radius_for_mass(mass),
                    },
                    SpriteBundle {
                        sprite: Sprite {
                            color: class.color(settings.color_palette),
                            custom_size: Some(Vec2::splat(Class::radius_for_mass(mass))),
                            ..default()
                        },
                        transform: Transform::from_translation(pos.extend(0.0)),
                        ..default()
                    },
                ));
                stats.0 += 1;
            }
        }
    }
}

pub fn spawn_initial_bodies(
    mut commands: Commands,
    mut stats: ResMut<SimStats>,
    settings: Res<SimSettings>,
) {
    spawn_initial_bodies_inner(&mut commands, stats.as_mut(), &settings);
}

pub fn spawn_player(mut commands: Commands) {
    let mass = 80.0;
    let class = Class::from_mass(mass);
    commands.spawn((
        Body {
            mass,
            vel: Vec2::new(0.0, 130.0),
            acc: Vec2::ZERO,
            class,
        },
        SmoothSize {
            target_radius: Class::radius_for_mass(mass),
        },
        Player {
            prev_class: class,
            score: 0.0,
        },
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.9, 1.0, 0.9),
                custom_size: Some(Vec2::splat(Class::radius_for_mass(mass) + 1.5)),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(340.0, 0.0, 0.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn kick1_drift(settings: Res<SimSettings>, mut q: Query<(&mut Body, &mut Transform)>) {
    if !settings.running {
        return;
    }
    let dt = settings.dt * settings.time_scale;

    for (mut b, mut t) in &mut q {
        // v_half = v + a * dt/2
        let v_half = b.vel + b.acc * dt * 0.5;

        // p = p + v_half * dt
        let p = t.translation.truncate() + v_half * dt;
        t.translation.x = p.x;
        t.translation.y = p.y;

        // Store intermediate velocity for the next kick.
        b.vel = v_half;
    }
}

fn rebuild_quadtree(mut tree: ResMut<TreeState>, q: Query<(&Body, &Transform)>) {
    let mut max_extent = tree.bounds.half_size;
    for (_, t) in &q {
        max_extent = max_extent.max(t.translation.truncate().abs().max_element());
    }
    let size = (max_extent * 1.2).max(2000.0);
    tree.bounds = Quad::new(Vec2::ZERO, size);

    let mut qt = QuadTree::new(tree.bounds);
    for (b, t) in &q {
        qt.insert(t.translation.truncate(), b.mass);
    }
    qt.build_mass_centers();
    tree.root = Some(qt);
}

fn apply_bh_forces(
    settings: Res<SimSettings>,
    mut q: Query<(Entity, &mut Body, &Transform)>,
    tree: Res<TreeState>,
) {
    if tree.root.is_none() || !settings.running {
        return;
    }
    let qt = tree.root.as_ref().unwrap();

    // snapshot positions
    let items: Vec<(Entity, Vec2)> = q
        .iter()
        .map(|(e, _, t)| (e, t.translation.truncate()))
        .collect();

    // compute accelerations
    let mut acc_map: HashMap<Entity, Vec2> = HashMap::with_capacity(items.len());
    for (e, pos) in items {
        let density = qt.get_density_factor(pos);

        let theta = if settings.adaptive_theta {
            // lerp(max, min, factor)
            settings.theta_range.y - density * (settings.theta_range.y - settings.theta_range.x)
        } else {
            settings.theta
        };

        let softening = if settings.adaptive_softening {
            // lerp(min, max, factor)
            settings.softening_range.x
                + density * (settings.softening_range.y - settings.softening_range.x)
        } else {
            settings.softening
        };
        let soft2 = softening * softening;

        acc_map.insert(e, qt.approx_acc(pos, settings.g, theta, soft2));
    }

    // write back acc
    for (e, mut b, _) in &mut q {
        if let Some(&acc) = acc_map.get(&e) {
            b.acc = acc;
        }
    }
}

fn kick2(settings: Res<SimSettings>, mut q: Query<&mut Body>) {
    if !settings.running {
        return;
    }
    let dt = settings.dt * settings.time_scale;

    for mut b in &mut q {
        // v = v_half + a * dt/2
        let v_full = b.vel + b.acc * dt * 0.5;
        b.vel = v_full.clamp_length_max(settings.max_vel);
    }
}

#[derive(Default)]
struct SpatialHash {
    cell: f32,
    map: HashMap<(i32, i32), Vec<Entity>>,
}

fn spatial_hash_build(mut hash: Local<SpatialHash>, q: Query<(Entity, &Transform, &Body)>) {
    let mut total_radius = 0.0;
    let mut count = 0;
    for (_, _, b) in &q {
        total_radius += Class::radius_for_mass(b.mass);
        count += 1;
    }

    hash.cell = if count > 0 {
        let mean_radius = total_radius / count as f32;
        (2.0 * mean_radius).max(1.0)
    } else {
        24.0 // Default
    };

    hash.map.clear();
    for (e, t, _b) in &q {
        let p = t.translation.truncate();
        let key = (
            (p.x / hash.cell).floor() as i32,
            (p.y / hash.cell).floor() as i32,
        );
        hash.map.entry(key).or_default().push(e);
    }
}

// Detect → queue → apply, with ParamSet to avoid B0001 aliasing.
fn resolve_collisions(
    mut commands: Commands,
    settings: Res<SimSettings>,
    mut stats: ResMut<SimStats>,
    mut q: ParamSet<(
        Query<(Entity, &Body, &Transform, Option<&Player>)>, // read-only
        Query<(Entity, &mut Body, &mut Transform)>,          // write-only
    )>,
    mut died: EventWriter<PlayerDied>,
    mut ev_absorbed: EventWriter<BodyAbsorbed>,
    hash: Local<SpatialHash>,
) {
    let neighbor_offsets = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (0, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let radius_of = |b: &Body| Class::radius_for_mass(b.mass);

    match settings.collision_mode {
        CollisionMode::Absorb => {
            #[derive(Clone)]
            struct Merge {
                winner: Entity,
                loser: Entity,
                new_mass: f32,
                new_vel: Vec2,
                player_died: bool,
            }

            let mut merges: Vec<Merge> = Vec::new();
            let mut removed: HashSet<Entity> = HashSet::new();

            {
                let q_read = q.p0();
                for (cell_key, ents) in hash.map.iter() {
                    let mut candidates: Vec<Entity> = Vec::new();
                    for off in neighbor_offsets {
                        let key = (cell_key.0 + off.0, cell_key.1 + off.1);
                        if let Some(v) = hash.map.get(&key) {
                            candidates.extend_from_slice(v);
                        }
                    }

                    for &a in ents {
                        if removed.contains(&a) {
                            continue;
                        }
                        let Ok((ea, ba, ta, pla)) = q_read.get(a) else {
                            continue;
                        };
                        let pa = ta.translation.truncate();
                        let ra = radius_of(ba);

                        for &b in &candidates {
                            if a == b || removed.contains(&b) {
                                continue;
                            }
                            let Ok((eb, bb, tb, plb)) = q_read.get(b) else {
                                continue;
                            };
                            let pb = tb.translation.truncate();
                            let rb = radius_of(bb);

                            let rsum = ra + rb;
                            if (pb - pa).length_squared() > rsum * rsum {
                                continue;
                            }

                            let a_is_bh = ba.class == Class::BlackHole;
                            let b_is_bh = bb.class == Class::BlackHole;
                            let a_wins = if a_is_bh && !b_is_bh {
                                true
                            } else if b_is_bh && !a_is_bh {
                                false
                            } else {
                                ba.mass >= bb.mass
                            };

                            let total = ba.mass + bb.mass;
                            let bias = 1.0 + settings.absorb_bias;

                            if a_wins {
                                let new_mass = (ba.mass * bias + bb.mass).max(ba.mass);
                                let new_vel = (ba.vel * ba.mass + bb.vel * bb.mass) / total;
                                merges.push(Merge {
                                    winner: ea,
                                    loser: eb,
                                    new_mass,
                                    new_vel,
                                    player_died: plb.is_some(),
                                });
                                ev_absorbed.send(BodyAbsorbed {
                                    winner: ea,
                                    loser_mass: bb.mass,
                                    loser_vel: bb.vel,
                                    loser_class: bb.class,
                                });
                                removed.insert(eb);
                            } else {
                                let new_mass = (bb.mass * bias + ba.mass).max(bb.mass);
                                let new_vel = (ba.vel * ba.mass + bb.vel * bb.mass) / total;
                                merges.push(Merge {
                                    winner: eb,
                                    loser: ea,
                                    new_mass,
                                    new_vel,
                                    player_died: pla.is_some(),
                                });
                                ev_absorbed.send(BodyAbsorbed {
                                    winner: eb,
                                    loser_mass: ba.mass,
                                    loser_vel: ba.vel,
                                    loser_class: ba.class,
                                });
                                removed.insert(ea);
                                break;
                            }
                        }
                    }
                }
            }

            let mut already_gone: HashSet<Entity> = HashSet::new();
            let mut q_write = q.p1();
            for m in merges {
                if already_gone.contains(&m.loser) {
                    continue;
                }
                if let Ok((_, mut bw, _)) = q_write.get_mut(m.winner) {
                    bw.mass = m.new_mass;
                    bw.class = Class::from_mass(bw.mass);
                    bw.vel = m.new_vel;
                } else {
                    continue;
                }
                if m.player_died {
                    died.send(PlayerDied);
                }
                if q_write.get_mut(m.loser).is_ok() {
                    commands.entity(m.loser).despawn_recursive();
                    already_gone.insert(m.loser);
                    stats.0 = stats.0.saturating_sub(1);
                }
            }
        }
        CollisionMode::Elastic => {
            struct ElasticResult {
                entity: Entity,
                new_vel: Vec2,
                new_pos: Vec2,
            }
            let mut updates: Vec<ElasticResult> = Vec::new();
            let mut processed: HashSet<Entity> = HashSet::new();

            {
                let q_read = q.p0();
                for (cell_key, ents) in hash.map.iter() {
                    let mut candidates: Vec<Entity> = Vec::new();
                    for off in neighbor_offsets {
                        let key = (cell_key.0 + off.0, cell_key.1 + off.1);
                        if let Some(v) = hash.map.get(&key) {
                            candidates.extend_from_slice(v);
                        }
                    }

                    for &a in ents {
                        if processed.contains(&a) {
                            continue;
                        }
                        let Ok((ea, ba, ta, _)) = q_read.get(a) else {
                            continue;
                        };
                        let pa = ta.translation.truncate();
                        let ra = radius_of(ba);

                        for &b in &candidates {
                            if a == b || processed.contains(&b) {
                                continue;
                            }
                            let Ok((eb, bb, tb, _)) = q_read.get(b) else {
                                continue;
                            };
                            let pb = tb.translation.truncate();
                            let rb = radius_of(bb);

                            let delta = pb - pa;
                            let dist2 = delta.length_squared();
                            let rsum = ra + rb;

                            if dist2 <= rsum * rsum && dist2 > 0.0 {
                                let dist = dist2.sqrt();
                                let normal = delta / dist;

                                let overlap = (rsum - dist) * 0.5;
                                let pa_new = pa - normal * overlap;
                                let pb_new = pb + normal * overlap;

                                let (va, vb) = (ba.vel, bb.vel);
                                let (ma, mb) = (ba.mass, bb.mass);
                                let tangent = Vec2::new(-normal.y, normal.x);
                                let van = va.dot(normal);
                                let vat = va.dot(tangent);
                                let vbn = vb.dot(normal);
                                let vbt = vb.dot(tangent);

                                let e = settings.restitution;
                                let van_new =
                                    (e * mb * (vbn - van) + ma * van + mb * vbn) / (ma + mb);
                                let vbn_new =
                                    (e * ma * (van - vbn) + ma * van + mb * vbn) / (ma + mb);

                                let va_new = van_new * normal + vat * tangent;
                                let vb_new = vbn_new * normal + vbt * tangent;

                                updates.push(ElasticResult {
                                    entity: ea,
                                    new_vel: va_new,
                                    new_pos: pa_new,
                                });
                                updates.push(ElasticResult {
                                    entity: eb,
                                    new_vel: vb_new,
                                    new_pos: pb_new,
                                });

                                processed.insert(ea);
                                processed.insert(eb);
                                break;
                            }
                        }
                    }
                }
            }

            let mut q_write = q.p1();
            for update in updates {
                if let Ok((_, mut body, mut trans)) = q_write.get_mut(update.entity) {
                    body.vel = update.new_vel;
                    trans.translation.x = update.new_pos.x;
                    trans.translation.y = update.new_pos.y;
                }
            }
        }
    }
}

fn update_render(
    mut q: Query<(&Body, &mut Sprite, &mut SmoothSize)>,
    time: Res<Time>,
    settings: Res<SimSettings>,
) {
    for (b, mut s, mut smooth_size) in &mut q {
        smooth_size.target_radius = Class::radius_for_mass(b.mass);

        let current_size = s
            .custom_size
            .unwrap_or(Vec2::splat(smooth_size.target_radius))
            .x;
        let lerp_factor = (1.0 - (-5.0 * time.delta_seconds()).exp()).clamp(0.0, 1.0);
        let new_size = current_size + (smooth_size.target_radius - current_size) * lerp_factor;

        s.custom_size = Some(Vec2::splat(new_size));

        let glow = b.class.glow();
        let linear_rgba: LinearRgba = b.class.color(settings.color_palette).into();
        let new_color: Color = (linear_rgba * glow).into();
        s.color = new_color;
    }
}

fn spawn_bursts(
    mut ev: EventReader<SpawnBurst>,
    mut commands: Commands,
    mut stats: ResMut<SimStats>,
    settings: Res<SimSettings>,
    mut seeded_rng: Option<ResMut<SeededRng>>,
) {
    let mut rng = rand::thread_rng();
    for e in ev.read() {
        if stats.0 >= settings.spawn_limit {
            continue;
        }

        let mut rng_source: Box<dyn RngCore> = if let Some(seeded) = seeded_rng.as_mut() {
            Box::new(&mut seeded.0)
        } else {
            Box::new(&mut rng)
        };

        let count = e.count.min(settings.spawn_limit - stats.0);
        for _ in 0..count {
            let r = rng_source.gen::<f32>() * e.radius;
            let ang = rng_source.gen::<f32>() * std::f32::consts::TAU;
            let offset = Vec2::from_angle(ang) * r;
            let pos = e.center + offset;
            let tangential = Vec2::new(-offset.y, offset.x).normalize_or_zero() * e.speed;
            let jitter = Vec2::new(
                rng_source.gen_range(-20.0..20.0),
                rng_source.gen_range(-20.0..20.0),
            );
            let mass = e.base_mass * rng_source.gen_range(0.5..1.5);
            let class = Class::from_mass(mass);
            commands.spawn((
                Body {
                    mass,
                    vel: tangential + jitter,
                    acc: Vec2::ZERO,
                    class,
                },
                SmoothSize {
                    target_radius: Class::radius_for_mass(mass),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: class.color(settings.color_palette),
                        custom_size: Some(Vec2::splat(Class::radius_for_mass(mass))),
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(0.0)),
                    ..default()
                },
            ));
        }
        stats.0 += count;
    }
}

fn handle_reset(
    mut commands: Commands,
    mut ev_reset: EventReader<ResetEvent>,
    body_q: Query<Entity, With<Body>>,
    mut stats: ResMut<SimStats>,
    mut settings: ResMut<SimSettings>,
) {
    if ev_reset.is_empty() {
        return;
    }
    ev_reset.clear();

    for e in &body_q {
        commands.entity(e).despawn_recursive();
    }
    stats.0 = 0;

    *settings = SimSettings::from_scenario(settings.scenario);
    spawn_initial_bodies_inner(&mut commands, stats.as_mut(), &*settings);
}

fn spawn_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TrailSpawnTimer>,
    settings: Res<SimSettings>,
    body_q: Query<(&Transform, &Body)>,
) {
    timer.0.tick(time.delta());
    if !settings.trails_enabled || !timer.0.just_finished() {
        return;
    }

    for (t, b) in &body_q {
        if b.vel.length_squared() > 100.0 {
            // Only spawn for moving bodies
            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_translation(t.translation),
                    sprite: Sprite {
                        color: b.class.color(settings.color_palette).with_alpha(0.5),
                        custom_size: Some(Vec2::splat(Class::radius_for_mass(b.mass) * 0.5)),
                        ..default()
                    },
                    ..default()
                },
                Trail {
                    lifespan: settings.trail_lifespan,
                },
            ));
        }
    }
}

fn update_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut trail_q: Query<(Entity, &mut Trail, &mut Sprite)>,
    settings: Res<SimSettings>,
) {
    let dt = time.delta_seconds();
    for (e, mut trail, mut sprite) in &mut trail_q {
        trail.lifespan -= dt;
        if trail.lifespan <= 0.0 {
            commands.entity(e).despawn();
        } else {
            let alpha = (trail.lifespan / settings.trail_lifespan).clamp(0.0, 1.0) * 0.5;
            sprite.color.set_alpha(alpha);
        }
    }
}

fn check_player_evolution(
    mut player_q: Query<(&Transform, &Body, &mut Player)>,
    mut ev_spawn: EventWriter<SpawnBurst>,
) {
    if let Ok((transform, body, mut player)) = player_q.get_single_mut() {
        if body.class != player.prev_class {
            player.prev_class = body.class;
            ev_spawn.send(SpawnBurst {
                center: transform.translation.truncate(),
                radius: Class::radius_for_mass(body.mass) * 1.5,
                count: 30,
                base_mass: 10.0,
                speed: 150.0,
            });
        }
    }
}

fn update_score(
    mut ev_absorbed: EventReader<BodyAbsorbed>,
    mut player_q: Query<(Entity, &mut Player)>,
) {
    if let Ok((player_entity, mut player)) = player_q.get_single_mut() {
        for ev in ev_absorbed.read() {
            if ev.winner == player_entity {
                let score_gain = (ev.loser_mass * ev.loser_vel.length()) / ev.loser_class.rarity();
                player.score += score_gain;
            }
        }
    }
}

fn spawn_hazards(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<HazardSpawnTimer>,
    mut ev_spawn: EventWriter<SpawnBurst>,
    settings: Res<SimSettings>,
    q_player: Query<&Transform, With<Player>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let player_pos = if let Ok(p) = q_player.get_single() {
        p.translation.truncate()
    } else {
        Vec2::ZERO
    };

    let mut rng = rand::thread_rng();
    let hazard_type = rng.gen_range(0..3);

    match hazard_type {
        0 => {
            // Rogue Star
            let pos = player_pos
                + Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero()
                    * 2000.0;
            let vel = (player_pos - pos).normalize() * 300.0;
            let mass = 100_000.0;
            let class = Class::from_mass(mass);
            commands.spawn((
                Body {
                    mass,
                    vel,
                    acc: Vec2::ZERO,
                    class,
                },
                SmoothSize {
                    target_radius: Class::radius_for_mass(mass),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: class.color(settings.color_palette),
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(0.0)),
                    ..default()
                },
                Hazard,
            ));
        }
        1 => {
            // Micro BH
            let pos = player_pos
                + Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero()
                    * 1500.0;
            let mass = 1_500_000.0;
            let class = Class::from_mass(mass);
            commands.spawn((
                Body {
                    mass,
                    vel: Vec2::ZERO,
                    acc: Vec2::ZERO,
                    class,
                },
                SmoothSize {
                    target_radius: Class::radius_for_mass(mass),
                },
                SpriteBundle {
                    sprite: Sprite {
                        color: class.color(settings.color_palette),
                        ..default()
                    },
                    transform: Transform::from_translation(pos.extend(0.0)),
                    ..default()
                },
                Hazard,
            ));
        }
        2 => {
            // Debris Storm
            let pos = player_pos
                + Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero()
                    * 3000.0;
            ev_spawn.send(SpawnBurst {
                center: pos,
                radius: 200.0,
                count: 100,
                base_mass: 20.0,
                speed: 400.0,
            });
        }
        _ => {}
    }
}

fn update_mission(mut mission: ResMut<Mission>, time: Res<Time>) {
    if mission.completed {
        return;
    }

    match mission.objective {
        Objective::Survive => {
            mission.progress += time.delta_seconds();
            if mission.progress >= mission.goal {
                mission.completed = true;
            }
        }
        Objective::None => {}
    }
}

fn player_death_system(
    mut ev_player_died: EventReader<PlayerDied>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if ev_player_died.read().next().is_some() {
        next_state.set(AppState::GameOver);
    }
}
