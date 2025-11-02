use bevy_egui::egui;

use crate::domain::simulation::{Mission, Objective};

pub fn show_mission_panel(ctx: &mut egui::Context, mission: &Mission) {
    egui::Area::new("mission_panel".into())
        .anchor(egui::Align2::CENTER_TOP, egui::Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(get_mission_text(mission));
        });
}

fn get_mission_text(mission: &Mission) -> String {
    if mission.completed {
        return "".to_string();
    }

    match mission.objective {
        Objective::Survive => {
            format!("Survive: {:.0} / {:.0}s", mission.progress, mission.goal)
        }
        Objective::None => "".to_string(),
    }
}
