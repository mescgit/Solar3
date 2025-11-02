use crate::domain::simulation::{
    AppState, Body, Mission, Player, SimSettings, SimState, SimStats,
};
use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};
mod panels;
use panels::banners::show_banners;
use panels::diagnostics_panel::show_diagnostics_panel;
use panels::help_panel::show_help_panel;
use panels::mission_panel::show_mission_panel;
use panels::settings_panel::show_settings_panel;
pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            .add_systems(Update, ui_system.run_if(in_state(AppState::Playing)))
            .add_systems(Update, show_banners);
    }
}
#[allow(clippy::too_many_arguments)]
fn ui_system(
    mut contexts: EguiContexts,
    mut settings: ResMut<SimSettings>,
    stats: Res<SimStats>,
    player_q: Query<(&Body, &Player)>,
    mut next_state: ResMut<NextState<SimState>>,
    diagnostics: Res<DiagnosticsStore>,
    mission: Res<Mission>,
) {
    show_settings_panel(
        contexts.ctx_mut(),
        &mut settings,
        &stats,
        &player_q,
        &mut next_state,
        &diagnostics,
    );
    show_help_panel(contexts.ctx_mut(), &settings);
    show_diagnostics_panel(contexts.ctx_mut(), &diagnostics, &settings);
    show_mission_panel(contexts.ctx_mut(), &mission);
}
