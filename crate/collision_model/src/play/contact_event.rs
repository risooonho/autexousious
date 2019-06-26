use amethyst::ecs::Entity;
use derive_new::new;
use shape_model::Volume;

use crate::config::Interaction;

/// Event indicating contact has occurred.
#[derive(Clone, Debug, PartialEq, new)]
pub struct ContactEvent {
    /// Entity with the interaction.
    pub from: Entity,
    /// Entity whose body was hit.
    pub to: Entity,
    /// Interaction of the collision.
    pub interaction: Interaction,
    /// Body that was hit.
    pub body: Volume,
}
