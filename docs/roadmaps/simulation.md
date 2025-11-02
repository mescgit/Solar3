# Simulation Engineer Roadmap

The simulation engineer owns `src/domain/simulation/` (including the private `quadtree.rs`) and exposes gameplay data to the rest of the game through the public types in `mod.rs`. The work is split into milestones that build toward the full v1.0 feature set.

## Milestone A — Core Stability & Fidelity
- [ ] Profile the Barnes–Hut solver at different body counts; tune theta/softening defaults (`SimSettings`).
- [ ] Implement energy drift telemetry and automated regression test to keep leapfrog integration stable.
- [ ] Extend collision handling with elastic mode momentum validation and restitution tuning.
- [ ] Harden hazard spawning logic with scenario-aware seeds (`Scenario` variants) and cap simultaneous hazards.

## Milestone B — Gameplay Systems
- [ ] Finish objectives pipeline (`Mission`) with progress evaluators for survive/absorb/evolve goals.
- [ ] Implement lose state triggers (`PlayerDied` event) and broadcast win/loss outcomes via `AppState` transitions.
- [ ] Add NPC behaviors (rogue stars, escorts) driven by deterministic RNG (`SeededRng`).
- [ ] Surface score multipliers, streaks, and leaderboard data through `SimStats`.

## Milestone C — Content Expansion
- [ ] Author scenario presets with parameter builders (spawner utilities for belts, binaries, nurseries).
- [ ] Add seed import/export APIs for persistence, including serialization helpers.
- [ ] Integrate save/load snapshots for body fields (`Body`, `Class`) and hazards.
- [ ] Prepare performance guardrails: benchmarking harness, jemallocator flag, and fallback sequential mode tuning.

## Collaboration Notes
- Keep cross-team surface area limited to the public structs/events already defined in `mod.rs`.
- Coordinate new data fields with the controls/UI engineers via shared enums or accessor methods instead of exposing internal systems.
- Provide documentation comments for any new API so downstream teams can adopt it without editing simulation internals.
