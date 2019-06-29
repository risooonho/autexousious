#![deny(missing_debug_implementations, missing_docs)] // kcov-ignore

//! Provides logic used during game play.

pub use crate::system::{
    CollisionDetectionSystem, ContactDetectionSystem, HitDetectionSystem,
    HitRepeatTrackersAugmentSystem, HitRepeatTrackersTickerSystem,
};

mod system;
