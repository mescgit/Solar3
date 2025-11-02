use crate::domain::simulation::{
    Body, CollisionMode, ColorPalette, Mission, Objective, Player, Scenario, SimSettings,
    SimState, SimStats, SystemType,
};
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::egui;

#[allow(clippy::too_many_arguments)]
pub fn show_settings_panel(
    ctx: &mut egui::Context,
    settings: &mut SimSettings,
    stats: &SimStats,
    player_q: &Query<(&Body, &Player)>,
    next_state: &mut NextState<SimState>,
    diagnostics: &DiagnosticsStore,
    mission: &Mission,
) {
    egui::Window::new("Settings").show(ctx, |ui| {
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
    });
}
