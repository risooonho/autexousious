use amethyst::assets::{AssetStorage, Loader};
use character_model::loaded::{CharacterCts, CharacterInputReactions};
use derivative::Derivative;

/// Resources needed to load a control transitions sequence.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct CtsLoaderParams<'s> {
    /// `Loader` to load assets.
    #[derivative(Debug = "ignore")]
    pub loader: &'s Loader,
    /// `CharacterInputReactions` assets.
    #[derivative(Debug = "ignore")]
    pub character_input_reactions_assets: &'s AssetStorage<CharacterInputReactions>,
    /// `CharacterCts` assets.
    #[derivative(Debug = "ignore")]
    pub character_cts_assets: &'s AssetStorage<CharacterCts>,
}
