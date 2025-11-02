# Presentation & UX Engineer Roadmap

The presentation engineer owns `src/domain/presentation/` and consumes simulation state through public resources and events. Responsibilities include HUD, menus, and overall visual cohesion.

## Milestone A — HUD & Overlay Polish
- [ ] Modularize egui windows into reusable panels (settings, help, diagnostics) with theming constants.
- [ ] Display mission objectives, timers, and success/failure banners driven by `Mission` and `AppState`.
- [ ] Surface player evolution feedback (class transitions, score popups) without mutating simulation data.
- [ ] Integrate screenshot/photo mode toggles triggered by presentation resources only.

## Milestone B — Menu Flow & Accessibility
- [ ] Build main menu, scenario select, and options panels while keeping logic inside presentation layer.
- [ ] Add colorblind and contrast settings wired to `SimSettings.color_palette` and future theme fields.
- [ ] Implement customizable HUD layout (movable windows) saved to presentation-specific resources.
- [ ] Provide controller/keyboard navigation for all egui screens.

## Milestone C — Polish & Feedback
- [ ] Implement particle trail visualization controls and connect to simulation toggles.
- [ ] Add mission debrief screen summarizing stats from `SimStats` and events raised during gameplay.
- [ ] Hook up audio triggers (via future audio module) without altering simulation internals.
- [ ] Prepare localization scaffolding by wrapping strings with translator helpers.

## Collaboration Notes
- Treat the simulation module as read-only—pull data via queries/resources and emit presentation-specific events back to controls if needed.
- Coordinate new display fields with the simulation engineer before exposing them in the UI, and document expected ranges.
- Share layout constants and keybind icons with the controls engineer to keep HUD instructions synchronized.
