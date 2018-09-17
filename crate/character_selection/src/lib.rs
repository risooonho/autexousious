#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! State where character selection takes place.

extern crate amethyst;
#[cfg(test)]
extern crate amethyst_test_support;
#[cfg(test)]
extern crate application_event;
#[cfg(test)]
extern crate asset_loading;
#[cfg(test)]
extern crate assets_test;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate derive_new;
extern crate game_input;
extern crate game_model;
#[cfg(test)]
extern crate loading;
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate map_loading;
#[cfg(test)]
extern crate object_loading;
extern crate object_model;
extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate typename;
#[macro_use]
extern crate typename_derive;

pub use character_selection::CharacterSelection;
pub use character_selection_bundle::CharacterSelectionBundle;
pub use character_selection_event::CharacterSelectionEvent;
pub use character_selection_state::{CharacterSelectionState, CharacterSelectionStateBuilder};
pub use character_selections::CharacterSelections;
pub use character_selections_state::CharacterSelectionsState;
pub use system::CharacterSelectionSystem;

mod character_selection;
mod character_selection_bundle;
mod character_selection_event;
mod character_selection_state;
mod character_selections;
mod character_selections_state;
mod system;
