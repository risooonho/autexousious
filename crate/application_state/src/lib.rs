#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

//! Provides the `AutexState` trait to simplify implementing `amethyst::State`.





#[macro_use]
extern crate derivative;
#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate log;


pub use crate::app_state::{AppState, AppStateBuilder};
pub use crate::autex_state::AutexState;
pub use crate::hook_fn::HookFn;
pub use crate::hookable_fn::HookableFn;

mod app_state;
mod autex_state;
mod hook_fn;
mod hookable_fn;
