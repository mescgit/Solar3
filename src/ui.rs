use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin};
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use crate::sim::{Body, Player, SimSettings, SimStats, CollisionMode, SimState, ColorPalette, Mission, Objective, ResetEvent, AppState, SystemType};

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
) {
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
                body.mass,
                body.class,
                player.score
            ));
        }

        ui.separator();

        if !mission.completed {
            match mission.objective {
                Objective::Survive => {
                    ui.label(format!(
                        "Survive: {:.0} / {:.0}s",
                        mission.progress,
                        mission.goal
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
                ui.selectable_value(
                    &mut settings.scenario,
                    Scenario::CalmBelts,
                    "Calm Belts",
                );
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
                ui.selectable_value(
                    &mut settings.scenario,
                    Scenario::BHArena,
                    "BH Arena",
                );
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
                ui.selectable_value(
                    &mut settings.system_type,
                    SystemType::Cluster,
                    "Cluster",
                );
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

        if ui.checkbox(&mut settings.deterministic, "Deterministic").changed() {
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
            ui.add(
                egui::Slider::new(&mut settings.theta_range.x, 0.0..=1.0).text("Theta Min"),
            );
            ui.add(
                egui::Slider::new(&mut settings.theta_range.y, 0.0..=2.0).text("Theta Max"),
            );
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
            if let Some(entity_count) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT) {
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