#![deny(missing_debug_implementations, missing_docs)] // kcov-ignore

//! Provides behaviour logic for states.

pub use crate::system::{
    StateCameraResetSystem, StateCameraResetSystemData, StateIdEventSystem, StateIdEventSystemData,
    StateItemSpawnSystem, StateItemSpawnSystemData, StateItemUiInputAugmentSystem,
    StateItemUiRectifySystem, StateItemUiRectifySystemData, StateUiSpawnSystem,
    StateUiSpawnSystemData,
};

mod system;
