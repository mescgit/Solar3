use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy_egui::egui;

use crate::domain::simulation::SimSettings;

pub fn show_diagnostics_panel(
    ctx: &mut egui::Context,
    diagnostics: &DiagnosticsStore,
    settings: &SimSettings,
) {
    if settings.show_diagnostics {
        egui::Window::new("Diagnostics").show(ctx, |ui| {
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
