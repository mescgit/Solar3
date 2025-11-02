use bevy_egui::egui;

use crate::domain::simulation::SimSettings;

pub fn show_help_panel(ctx: &mut egui::Context, settings: &SimSettings) {
    if settings.show_help {
        egui::Window::new("Help").show(ctx, |ui| {
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
}
