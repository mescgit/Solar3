# Controls & Systems Engineer Roadmap

The controls engineer owns `src/domain/controls/` and interacts with the simulation solely through the public API in `crate::domain::simulation`. Focus on player input, camera, and accessibility of control schemes.

## Milestone A — Player Agency
- [ ] Expand thrust system with analog gamepad support (`player_thrust`).
- [ ] Implement configurable keybinds stored in a resource mirrored to UI sliders/toggles.
- [ ] Add contextual actions (target lock, dash) that emit new events for the simulation layer.
- [ ] Ensure camera controls adapt to both mouse/keyboard and controller input, with invert-Y toggle.

## Milestone B — Quality-of-Life Systems
- [ ] Build input rebinding screen data model; emit `ResetEvent` and future mission events in a consistent pattern.
- [ ] Implement tutorial guidance triggers (e.g., highlight controls when idle) without mutating simulation internals.
- [ ] Add pause overlay integration hooks by toggling `AppState` while keeping UI decoupled.
- [ ] Provide configurable assist modes (auto-follow strength, dampening) exposed via `SimSettings` fields.

## Milestone C — Persistence & Testing
- [ ] Serialize control profiles to disk and reload them on startup.
- [ ] Author integration tests for input pipelines using Bevy's schedule runner.
- [ ] Instrument diagnostics toggles to feed structured telemetry into the presentation layer.
- [ ] Document public helper functions in this module so other teams avoid direct mutations of internal state.

## Collaboration Notes
- Treat `SimSettings` and the event types (`SpawnBurst`, `ResetEvent`) as the primary integration points.
- Coordinate any new settings fields with the simulation engineer before committing so state transitions remain deterministic.
- Keep UI-specific logic in callbacks/events instead of editing egui code directly—hand off new data via resources.
