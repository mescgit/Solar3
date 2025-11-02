use crate::domain::simulation::{AppState, Mission};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, RichText},
    EguiContexts,
};

pub fn show_banners(
    mut contexts: EguiContexts,
    mission: Res<Mission>,
    app_state: Res<State<AppState>>,
) {
    match app_state.get() {
        AppState::GameOver => {
            show_failure_banner(contexts.ctx_mut());
        }
        AppState::Playing => {
            if mission.completed {
                show_success_banner(contexts.ctx_mut());
            }
        }
    }
}

fn show_success_banner(ctx: &mut egui::Context) {
    egui::Area::new("success_banner".into())
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            let text = RichText::new("Mission Completed!")
                .font(FontId::proportional(48.0))
                .color(Color32::GREEN);
            ui.label(text);
        });
}

fn show_failure_banner(ctx: &mut egui::Context) {
    egui::Area::new("failure_banner".into())
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            let text = RichText::new("Mission Failed!")
                .font(FontId::proportional(48.0))
                .color(Color32::RED);
            ui.label(text);
        });
}
