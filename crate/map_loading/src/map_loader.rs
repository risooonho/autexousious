use std::path::Path;

use amethyst::{assets::Loader, prelude::*, renderer::MaterialTextureSet};
use application::{load_in, ErrorKind, Format, Result};
use map_model::{
    config::MapDefinition,
    loaded::{Map, MapHandle, Margins},
};
use sprite_loading::{MaterialAnimationLoader, SpriteLoader};

/// Loads assets specified by map configuration into the loaded map model.
#[derive(Debug)]
pub struct MapLoader;

impl MapLoader {
    /// Returns the loaded `Map` referenced by the configuration record.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to store the map's assets.
    /// * `base_dir`: Base directory from which to load the map.
    pub fn load(world: &World, base_dir: &Path) -> Result<MapHandle> {
        debug!("Loading map in `{}`", base_dir.display());

        let map_definition = load_in::<MapDefinition, _>(base_dir, "map.toml", Format::Toml, None)?;
        let texture_index_offset = world.read_resource::<MaterialTextureSet>().len() as u64;

        let sprite_load_result = SpriteLoader::load(world, texture_index_offset, base_dir);
        let sprite_info = match sprite_load_result {
            Ok(sprite_info) => Ok(Some(sprite_info)),
            Err(e) => match e.kind() {
                ErrorKind::Find(..) => Ok(None),
                _ => Err(e),
            },
        }?;

        let (sprite_material_mesh, animation_handles) = {
            if let Some((sprite_sheets, sprite_material_mesh)) = sprite_info {
                let animation_handles = MaterialAnimationLoader::load_into_vec(
                    world,
                    map_definition.layers.iter(),
                    texture_index_offset,
                    &sprite_sheets,
                );
                (Some(sprite_material_mesh), Some(animation_handles))
            } else {
                (None, None)
            }
        };

        let margins = Margins::from(map_definition.header.bounds);

        let map = Map::new(
            map_definition,
            margins,
            sprite_material_mesh,
            animation_handles,
        );

        let loader = world.read_resource::<Loader>();
        let map_handle = loader.load_from_data(map, (), &world.read_resource());
        Ok(map_handle)
    }
}

// Covered by `MapLoadingBundle` test