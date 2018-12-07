#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! UI to allow the user to select the game mode.





#[macro_use]
extern crate derivative;
#[macro_use]
extern crate derive_new;


#[macro_use]
extern crate log;

use typename;
#[macro_use]
extern crate typename_derive;

pub use crate::game_mode_selection_ui_build_fn::GameModeSelectionUiBuildFn;
pub use crate::game_mode_selection_ui_bundle::GameModeSelectionUiBundle;
pub(crate) use crate::system::UiEventHandlerSystem;

mod game_mode_selection_ui_build_fn;
mod game_mode_selection_ui_bundle;
mod system;
