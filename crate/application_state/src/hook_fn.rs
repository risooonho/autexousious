use amethyst::ecs::World;
use derivative::Derivative;
use derive_deref::{Deref, DerefMut};

/// Wrapper for functions hooked into the `AppState`.
#[derive(Derivative, Deref, DerefMut)]
#[derivative(Debug)]
pub struct HookFn(#[derivative(Debug = "ignore")] pub fn(&mut World));
