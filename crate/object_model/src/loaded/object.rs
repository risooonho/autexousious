use amethyst::assets::Handle;
use amethyst::renderer::{Material, MeshHandle};
use amethyst_animation::Animation;

/// Represents an in-game object that has been loaded.
#[derive(Constructor, Clone, Derivative)]
#[derivative(Debug)]
pub struct Object {
    /// Default material for entities of this object.
    ///
    /// Even though practically entities will be displayed with a certain animation at all times,
    /// Amethyst requires us to set a default material for entities. If we don't then it panics.
    #[derivative(Debug = "ignore")]
    pub default_material: Material,
    /// Handle to the mesh to map the sprite texture to the screen.
    pub mesh: MeshHandle,
    /// Handle to the animations that this object uses.
    ///
    /// This should be substituted with `loaded::Sequences` which will contain the animations.
    pub animations: Vec<Handle<Animation<Material>>>,
}