use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::domain::controls::Keybinds;
use crate::domain::simulation::{
    AppState, Body, CollisionMode, ColorPalette, Mission, Objective, Player, ResetEvent, Scenario,
    SimSettings, SimState, SimStats, SystemType,
};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, ui_system.run_if(in_state(AppState::Playing)))
            .add_systems(Update, game_over_ui.run_if(in_state(AppState::GameOver)));
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<SimSettings>,
    stats: Res<SimStats>,
    player_q: Query<(&Body, &Player)>,
    mut next_state: ResMut<NextState<SimState>>,
    diagnostics: Res<DiagnosticsStore>,
    mission: Res<Mission>,
    mut keybinds: ResMut<Keybinds>,
    mut rebinding_state: Local<Option<String>>,
) {
    let mut pressed_key = None;
    contexts.ctx_mut().input(|i| {
        for event in &i.events {
            if let egui::Event::Key {
                key,
                pressed: true,
                ..
            } = event
            {
                pressed_key = Some(*key);
            }
        }
    });

    egui::Window::new("Settings").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Bodies: {}", stats.0));
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                ui.label(format!("FPS: {:.1}", value));
            }
        }
        ui.label(format!("Sim Rate: {:.2}x", settings.time_scale));
        if let Ok((body, player)) = player_q.get_single() {
            ui.label(format!(
                "Player — Mass: {:.1}  Class: {:?}  Score: {:.0}",
                body.mass, body.class, player.score
            ));
        }

        ui.separator();

        if !mission.completed {
            match mission.objective {
                Objective::Survive => {
                    ui.label(format!(
                        "Survive: {:.0} / {:.0}s",
                        mission.progress, mission.goal
                    ));
                }
                _ => {}
            }
        } else {
            ui.label("Mission Completed!");
        }

        ui.separator();

        ui.checkbox(&mut settings.running, "Running");
        ui.add(egui::Slider::new(&mut settings.g, 0.0..=500.0).text("Gravity (G)"));
        ui.add(egui::Slider::new(&mut settings.dt, 0.001..=0.03).text("Timestep (dt)"));

        ui.separator();

        egui::ComboBox::from_label("Scenario")
            .selected_text(format!("{:?}", settings.scenario))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut settings.scenario, Scenario::CalmBelts, "Calm Belts");
                ui.selectable_value(
                    &mut settings.scenario,
                    Scenario::BinaryMayhem,
                    "Binary Mayhem",
                );
                ui.selectable_value(
                    &mut settings.scenario,
                    Scenario::StarNursery,
                    "Star Nursery",
                );
                ui.selectable_value(&mut settings.scenario, Scenario::BHArena, "BH Arena");
            });

        ui.separator();

        egui::ComboBox::from_label("System Type")
            .selected_text(format!("{:?}", settings.system_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut settings.system_type,
                    SystemType::SingleStar,
                    "Single Star",
                );
                ui.selectable_value(
                    &mut settings.system_type,
                    SystemType::BinaryStar,
                    "Binary Star",
                );
                ui.selectable_value(&mut settings.system_type, SystemType::Cluster, "Cluster");
            });

        ui.separator();

        egui::ComboBox::from_label("Collision Mode")
            .selected_text(format!("{:?}", settings.collision_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut settings.collision_mode,
                    CollisionMode::Absorb,
                    "Absorb",
                );
                ui.selectable_value(
                    &mut settings.collision_mode,
                    CollisionMode::Elastic,
                    "Elastic",
                );
            });
        ui.add(egui::Slider::new(&mut settings.restitution, 0.0..=1.0).text("Restitution"));

        ui.separator();

        if ui
            .checkbox(&mut settings.deterministic, "Deterministic")
            .changed()
        {
            if settings.deterministic {
                next_state.set(SimState::Sequential);
            } else {
                next_state.set(SimState::Parallel);
            }
        }
        ui.checkbox(&mut settings.trails_enabled, "Trails");

        ui.separator();

        egui::ComboBox::from_label("Color Palette")
            .selected_text(format!("{:?}", settings.color_palette))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut settings.color_palette,
                    ColorPalette::Default,
                    "Default",
                );
                ui.selectable_value(
                    &mut settings.color_palette,
                    ColorPalette::Colorblind,
                    "Colorblind",
                );
            });

        ui.separator();

        ui.checkbox(&mut settings.adaptive_theta, "Adaptive Theta");
        if settings.adaptive_theta {
            ui.add(egui::Slider::new(&mut settings.theta_range.x, 0.0..=1.0).text("Theta Min"));
            ui.add(egui::Slider::new(&mut settings.theta_range.y, 0.0..=2.0).text("Theta Max"));
        } else {
            ui.add(egui::Slider::new(&mut settings.theta, 0.0..=2.0).text("Theta"));
        }

        ui.separator();

        ui.checkbox(&mut settings.adaptive_softening, "Adaptive Softening");
        if settings.adaptive_softening {
            ui.add(
                egui::Slider::new(&mut settings.softening_range.x, 0.1..=10.0)
                    .text("Softening Min"),
            );
            ui.add(
                egui::Slider::new(&mut settings.softening_range.y, 0.1..=20.0)
                    .text("Softening Max"),
            );
        } else {
            ui.add(egui::Slider::new(&mut settings.softening, 0.1..=20.0).text("Softening"));
        }

        ui.separator();

        ui.label("Keybinds");

        let mut rebind_action = |ui: &mut egui::Ui, action: &str, key: &mut KeyCode| {
            ui.horizontal(|ui| {
                ui.label(action);
                let button_text = if rebinding_state.as_deref() == Some(action) {
                    "Press a key..."
                } else {
                    &format!("{:?}", key)
                };
                if ui.button(button_text).clicked() {
                    *rebinding_state = Some(action.to_string());
                }
            });
        };

        rebind_action(ui, "Up", &mut keybinds.up);
        rebind_action(ui, "Down", &mut keybinds.down);
        rebind_action(ui, "Left", &mut keybinds.left);
        rebind_action(ui, "Right", &mut keybinds.right);
        rebind_action(ui, "Boost", &mut keybinds.boost);

        if let Some(action) = rebinding_state.take() {
            if let Some(key_code) = pressed_key {
                let bevy_keycode = egui_to_bevy_keycode(key_code);
                match action.as_str() {
                    "Up" => keybinds.up = bevy_keycode,
                    "Down" => keybinds.down = bevy_keycode,
                    "Left" => keybinds.left = bevy_keycode,
                    "Right" => keybinds.right = bevy_keycode,
                    "Boost" => keybinds.boost = bevy_keycode,
                    _ => {}
                }
            } else {
                *rebinding_state = Some(action);
            }
        }
    });

    if settings.show_help {
        egui::Window::new("Help").show(contexts.ctx_mut(), |ui| {
            ui.label("WASD/Arrows: Thrust");
            ui.label("Shift: Boost");
            ui.label("F: Toggle Camera Follow");
            ui.label("Space: Pause Simulation");
            ui.label("[/]: Adjust Sim Speed");
            ui.label("R: Reset Simulation");
            ui.label("H: Toggle Help");
            ui.label("Left Mouse: Spawn Burst (drag)");
            ui.label("Right Mouse: Pan Camera (drag)");
            ui.label("Mouse Wheel: Zoom");
        });
    }

    if settings.show_diagnostics {
        egui::Window::new("Diagnostics").show(contexts.ctx_mut(), |ui| {
            if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(value) = fps.smoothed() {
                    ui.label(format!("FPS: {:.1}", value));
                }
            }
            if let Some(entity_count) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
            {
                if let Some(value) = entity_count.value() {
                    ui.label(format!("Entities: {}", value));
                }
            }
        });
    }
}

fn game_over_ui(
    mut contexts: EguiContexts,
    mut ev_reset: EventWriter<ResetEvent>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    egui::Window::new("Game Over").show(contexts.ctx_mut(), |ui| {
        ui.label("You Died!");
        if ui.button("Retry").clicked() {
            ev_reset.send(ResetEvent::default());
            next_state.set(AppState::Playing);
        }
    });
}

fn egui_to_bevy_keycode(key: egui::Key) -> KeyCode {
    match key {
        egui::Key::A => KeyCode::KeyA,
        egui::Key::B => KeyCode::KeyB,
        egui::Key::C => KeyCode::KeyC,
        egui::Key::D => KeyCode::KeyD,
        egui::Key::E => KeyCode::KeyE,
        egui::Key::F => KeyCode::KeyF,
        egui::Key::G => KeyCode::KeyG,
        egui::Key::H => KeyCode::KeyH,
        egui::Key::I => KeyCode::KeyI,
        egui::Key::J => KeyCode::KeyJ,
        egui::Key::K => KeyCode::KeyK,
        egui::Key::L => KeyCode::KeyL,
        egui::Key::M => KeyCode::KeyM,
        egui::Key::N => KeyCode::KeyN,
        egui::Key::O => KeyCode::KeyO,
        egui::Key::P => KeyCode::KeyP,
        egui::Key::Q => KeyCode::KeyQ,
        egui::Key::R => KeyCode::KeyR,
        egui::Key::S => KeyCode::KeyS,
        egui::Key::T => KeyCode::KeyT,
        egui::Key::U => KeyCode::KeyU,
        egui::Key::V => KeyCode::KeyV,
        egui::Key::W => KeyCode::KeyW,
        egui::Key::X => KeyCode::KeyX,
        egui::Key::Y => KeyCode::KeyY,
        egui::Key::Z => KeyCode::KeyZ,
        egui::Key::Num0 => KeyCode::Digit0,
        egui::Key::Num1 => KeyCode::Digit1,
        egui::Key::Num2 => KeyCode::Digit2,
        egui::Key::Num3 => KeyCode::Digit3,
        egui::Key::Num4 => KeyCode::Digit4,
        egui::Key::Num5 => KeyCode::Digit5,
        egui::Key::Num6 => KeyCode::Digit6,
        egui::Key::Num7 => KeyCode::Digit7,
        egui::Key::Num8 => KeyCode::Digit8,
        egui::Key::Num9 => KeyCode::Digit9,
        egui::Key::ArrowUp => KeyCode::ArrowUp,
        egui::Key::ArrowDown => KeyCode::ArrowDown,
        egui::Key::ArrowLeft => KeyCode::ArrowLeft,
        egui::Key::ArrowRight => KeyCode::ArrowRight,
        egui::Key::Space => KeyCode::Space,
        egui::Key::Enter => KeyCode::Enter,
        egui::Key::Alt => KeyCode::AltLeft,
        egui::Key::Control => KeyCode::ControlLeft,
        egui::Key::Shift => KeyCode::ShiftLeft,
        _ => KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
    }
}
