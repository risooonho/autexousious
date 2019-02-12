use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    renderer::{SpriteSheet, SpriteSheetHandle, TextureHandle},
};
use sprite_model::config::SpriteSheetDefinition;

use crate::SpriteSheetMapper;

#[derive(Debug)]
pub(crate) struct SpriteSheetLoader;

impl SpriteSheetLoader {
    /// Loads Amethyst `SpriteSheet`s from configuration and returns their handles.
    ///
    /// # Parameters
    ///
    /// * `progress_counter`: `ProgressCounter` to track loading.
    /// * `loader`: `Loader` to load assets.
    /// * `sprite_sheet_assets`: `AssetStorage` for `SpriteSheet`s.
    /// * `texture_handles`: Handles of the sprite sheets' textures.
    /// * `sprite_sheet_definitions`: List of metadata for sprite sheets to map.
    pub fn load(
        progress_counter: &mut ProgressCounter,
        loader: &Loader,
        sprite_sheet_assets: &AssetStorage<SpriteSheet>,
        texture_handles: &[TextureHandle],
        sprite_sheet_definitions: &[SpriteSheetDefinition],
    ) -> Vec<SpriteSheetHandle> {
        let sprite_sheets = SpriteSheetMapper::map(texture_handles, &sprite_sheet_definitions);

        sprite_sheets
            .into_iter()
            .map(|sprite_sheet| {
                loader.load_from_data(sprite_sheet, &mut *progress_counter, sprite_sheet_assets)
            })
            .collect::<Vec<_>>()
    }
}
