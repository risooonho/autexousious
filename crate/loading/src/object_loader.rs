use std::fs::File;
use std::io::prelude::*;

use amethyst::assets::{AssetStorage, Loader};
use amethyst::prelude::*;
use amethyst::renderer::{Material, MaterialDefaults, SpriteSheet, SpriteSheetHandle, TextureHandle};
use amethyst_animation::MaterialTextureSet;
use game_model::ConfigRecord;
use object_model::config::SpritesDefinition;
use object_model::loaded;
use toml;

use error::Result;
use sprite::into_sprite_sheet;
use texture;

pub struct ObjectLoader<'w> {
    /// The world in which to load object assets.
    pub world: &'w World,
}

impl<'w> ObjectLoader<'w> {
    pub fn load_object(&mut self, config_record: &ConfigRecord) -> Result<loaded::Object> {
        let sprites_path = config_record.directory.join("sprites.toml");
        let mut sprites_toml = File::open(sprites_path)?;
        let mut buffer = Vec::new();
        sprites_toml.read_to_end(&mut buffer)?;

        let sprites_definition = toml::from_slice::<SpritesDefinition>(&buffer)?;
        let sprite_sheet_handles = sprites_definition
            .sheets
            .iter()
            .enumerate() // TODO: Pass in a calculated sprite index
            .map(into_sprite_sheet)
            .map(|sprite_sheet| {
                // Store the sprite sheet in asset storage.
                let loader = self.world.read_resource::<Loader>();
                loader.load_from_data(
                    sprite_sheet,
                    (),
                    &self.world.read_resource::<AssetStorage<SpriteSheet>>(),
                )
            })
            .collect::<Vec<SpriteSheetHandle>>();

        let texture_handles = sprites_definition
            .sheets
            .into_iter()
            .map(|sheet_definition| texture::load(sheet_definition.path, &self.world))
            .collect::<Vec<TextureHandle>>();

        // Create default material for the object
        let default_material = {
            let mat_defaults = self.world.read_resource::<MaterialDefaults>();
            texture_handles.first().map_or_else(
                || mat_defaults.0.clone(),
                |first_texture| Material {
                    albedo: first_texture.clone(),
                    ..mat_defaults.0.clone()
                },
            )
        };

        // TODO: Load animations.

        // TODO: Use calculated sprite index when registering sprite sheet texture
        texture_handles
            .into_iter()
            .enumerate()
            .for_each(|(index, texture_handle)| {
                self.world
                    .write_resource::<MaterialTextureSet>()
                    .insert(index, texture_handle);
            });

        // TODO: Swap sprite_sheet_handles for animation handles
        Ok(loaded::Object::new(default_material, sprite_sheet_handles))
    }
}
