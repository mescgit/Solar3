use crate::sim::{Body, Player, ResetEvent, SimSettings, SpawnBurst};
use crate::MainCamera;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::input::ButtonState; // needed in Bevy 0.14
use bevy::prelude::*;

pub struct InputPlugin;
impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DragState::default()).add_systems(
            Update,
            (
                camera_controls,
                drag_spawn,
                player_thrust,
                pause_toggle,
                follow_toggle,
                time_scale_toggle,
                reset_trigger,
                help_toggle,
                diagnostics_toggle,
            ),
        );
    }
}

#[derive(Resource, Default)]
struct DragState {
    start: Option<Vec2>,
    current: Vec2,
    button: Option<MouseButton>,
}

fn window_cursor_world(
    _window: &Window,
    cursor_pos: Vec2,
    cam: (&Camera, &GlobalTransform),
) -> Option<Vec2> {
    cam.0.viewport_to_world_2d(cam.1, cursor_pos)
}

fn camera_controls(
    mut scroll_evr: EventReader<MouseWheel>,
    mut q_cam: Query<(&mut Transform, &Camera, &GlobalTransform), With<MainCamera>>,
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<bevy::input::mouse::MouseMotion>,
    player_q: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    settings: Res<SimSettings>,
    time: Res<Time>,
) {
    let (mut t, cam, g_transform) = q_cam.single_mut();
    let win = windows.single();

    // Zoom to cursor
    if let Some(cursor_pos) = win.cursor_position() {
        if let Some(cursor_world_pos) = cam.viewport_to_world_2d(g_transform, cursor_pos) {
            for ev in scroll_evr.read() {
                let zoom = 1.0 - ev.y * 0.05;
                let new_scale = (t.scale * zoom).clamp(Vec3::splat(0.2), Vec3::splat(10.0));
                let actual_zoom = new_scale.x / t.scale.x;

                if (actual_zoom - 1.0).abs() > 1e-4 {
                    t.translation.x =
                        cursor_world_pos.x + (t.translation.x - cursor_world_pos.x) * actual_zoom;
                    t.translation.y =
                        cursor_world_pos.y + (t.translation.y - cursor_world_pos.y) * actual_zoom;
                    t.scale = new_scale;
                }
            }
        }
    }

    // Panning
    let mut is_panning = false;
    if buttons.pressed(MouseButton::Right) {
        for m in motion.read() {
            t.translation.x -= m.delta.x * t.scale.x;
            t.translation.y += m.delta.y * t.scale.y;
            is_panning = true;
        }
    }

    // Follow player
    if settings.follow_player && !is_panning {
        if let Ok(player_transform) = player_q.get_single() {
            let player_pos = player_transform.translation;
            let camera_pos = t.translation;
            let lerp_factor = (1.0 - (-2.0 * time.delta_seconds()).exp()).clamp(0.0, 1.0);
            let target_pos = player_pos.truncate();
            let new_pos = camera_pos.truncate().lerp(target_pos, lerp_factor);
            t.translation.x = new_pos.x;
            t.translation.y = new_pos.y;
        }
    }

    t.translation.z = 999.0;
}

fn drag_spawn(
    windows: Query<&Window>,
    q_cam: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut drag: ResMut<DragState>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut ev_spawn: EventWriter<SpawnBurst>,
    _settings: Res<SimSettings>,
) {
    let win = windows.single();
    let Some(cursor) = win.cursor_position() else {
        return;
    };
    let Some(world) = window_cursor_world(win, cursor, q_cam.single()) else {
        return;
    };

    for ev in mousebtn_evr.read() {
        match ev.state {
            ButtonState::Pressed if ev.button == MouseButton::Left => {
                drag.start = Some(world);
                drag.button = Some(MouseButton::Left);
            }
            ButtonState::Released if ev.button == MouseButton::Left => {
                if let Some(s) = drag.start.take() {
                    let radius = (world - s).length().max(10.0);
                    ev_spawn.send(SpawnBurst {
                        center: s,
                        radius,
                        count: (radius * 0.8) as usize,
                        base_mass: 20.0,
                        speed: 120.0,
                    });
                }
                drag.button = None;
            }
            _ => {}
        }
    }

    if buttons.pressed(MouseButton::Left) {
        drag.current = world;
    }
}

fn player_thrust(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<&mut Body, With<Player>>,
) {
    let dt = time.delta_seconds();
    if let Ok(mut player_body) = players.get_single_mut() {
        let mut dir = Vec2::ZERO;

        if keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW) {
            dir.y += 1.0;
        }
        if keys.pressed(KeyCode::ArrowDown) || keys.pressed(KeyCode::KeyS) {
            dir.y -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }

        if dir != Vec2::ZERO {
            let boost = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
                1.75
            } else {
                1.0
            };
            let acc = dir.normalize() * 380.0 * boost / player_body.mass.max(1.0);
            player_body.vel += acc * dt;
        }
    }
}

fn pause_toggle(mut settings: ResMut<SimSettings>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        settings.running = !settings.running;
    }
}

fn follow_toggle(mut settings: ResMut<SimSettings>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyF) {
        settings.follow_player = !settings.follow_player;
    }
}

fn time_scale_toggle(mut settings: ResMut<SimSettings>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::BracketRight) {
        settings.time_scale *= 2.0;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        settings.time_scale /= 2.0;
    }
    settings.time_scale = settings.time_scale.clamp(0.5, 4.0);
}

fn reset_trigger(mut ev_reset: EventWriter<ResetEvent>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyR) {
        ev_reset.send(ResetEvent::default());
    }
}

fn help_toggle(mut settings: ResMut<SimSettings>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyH) {
        settings.show_help = !settings.show_help;
    }
}

fn diagnostics_toggle(mut settings: ResMut<SimSettings>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::F3) {
        settings.show_diagnostics = !settings.show_diagnostics;
    }
}
