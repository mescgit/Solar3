use bevy::prelude::*;
use bevy_egui::egui;

use crate::domain::simulation::{AppState, ResetEvent};

pub fn show_game_over_panel(
    ctx: &mut egui::Context,
    ev_reset: &mut EventWriter<ResetEvent>,
    next_state: &mut NextState<AppState>,
) {
    egui::Window::new("Game Over").show(ctx, |ui| {
        ui.label("You Died!");
        if ui.button("Retry").clicked() {
            ev_reset.send(ResetEvent::default());
            next_state.set(AppState::Playing);
        }
    });
}
