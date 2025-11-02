//! Domain modules split by discipline so teams can work independently.
//! - `controls`: input, camera, and game flow toggles.
//! - `presentation`: HUD, menus, and UX overlays.
//! - `simulation`: physics, content, and authoritative game state.

pub mod controls;
pub mod presentation;
pub mod simulation;

pub use controls::InputPlugin;
pub use presentation::UiPlugin;
pub use simulation::{AppState, SimPlugin, SimState};
