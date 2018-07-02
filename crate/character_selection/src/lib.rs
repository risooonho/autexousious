#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Menu to allow the user to select game mode.

extern crate amethyst;
#[cfg(test)]
extern crate amethyst_test_support;
#[macro_use]
extern crate derivative;
#[macro_use]
extern crate derive_new;
extern crate game_input;
extern crate game_model;
#[macro_use]
extern crate log;
extern crate object_model;

pub use character_entity_control::CharacterEntityControl;
pub use character_selection::CharacterSelection;
pub use character_selection_bundle::CharacterSelectionBundle;
pub use character_selection_state::CharacterSelectionState;
pub(crate) use character_selection_system::CharacterSelectionSystem;

mod character_entity_control;
mod character_selection;
mod character_selection_bundle;
mod character_selection_state;
mod character_selection_system;
