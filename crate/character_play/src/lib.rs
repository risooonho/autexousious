#![deny(missing_debug_implementations, missing_docs)] // kcov-ignore

//! Behaviour logic for object types.

pub use crate::{
    character_sequence_update_components::CharacterSequenceUpdateComponents,
    character_sequence_updater::CharacterSequenceUpdater,
    mirrored_updater::MirroredUpdater,
    run_counter_updater::RunCounterUpdater,
    system::{CharacterInputReactionsTransitionSystem, CharacterInputReactionsUpdateSystem},
    system_data::ControlTransitionRequirementSystemData,
};

pub mod sequence_handler;

mod character_sequence_update_components;
mod character_sequence_updater;
mod mirrored_updater;
mod run_counter_updater;
mod system;
mod system_data;
