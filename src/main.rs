mod input;
mod quadtree;
mod sim;
mod ui;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use input::InputPlugin;
use sim::{AppState, SimPlugin, SimState};
use ui::UiPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .insert_resource(Msaa::Sample4)
        .init_state::<SimState>()
        .init_state::<AppState>()
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(EntityCountDiagnosticsPlugin)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "solar2-rs — n-body gravity".into(),
                resolution: (1400., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((SimPlugin, UiPlugin, InputPlugin))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. Enable HDR
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Add tonemapping
            transform: Transform::from_xyz(0.0, 0.0, 999.0).with_scale(Vec3::splat(1.0)),
            ..default()
        },
        BloomSettings::default(), // 3. Add bloom settings
        MainCamera,
    ));
}

#[derive(Component)]
pub struct MainCamera;
