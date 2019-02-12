use std::{collections::HashMap, path::Path};

use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    ecs::World,
    renderer::{SpriteSheet, Texture},
};
use application::{load_in, resource::Format};
use asset_loading::AssetDiscovery;
use asset_model::config::AssetRecord;
use assets_built_in::{MAP_BLANK, MAP_BLANK_SLUG};
use character_loading::CharacterLoader;
use game_model::loaded::{CharacterAssets, MapAssets};
use log::{debug, error};
use map_loading::MapLoader;
use map_model::loaded::MapHandle;
use object_model::ObjectType;
use sprite_loading::SpriteLoader;
use sprite_model::config::SpritesDefinition;
use strum::IntoEnumIterator;

/// Provides functions to load game assets.
#[derive(Debug)]
pub struct AssetLoader;

impl AssetLoader {
    /// Loads game assets into the `World` from the specified assets directory.
    ///
    /// When this function returns, the `World` will be populated with the `CharacterAssets` and
    /// `MapAssets` resources.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load the game assets into.
    /// * `progress_counter`: Tracker for loading progress.
    /// * `assets_dir`: Base directory containing all assets to load.
    pub fn load(world: &mut World, progress_counter: &mut ProgressCounter, assets_dir: &Path) {
        let asset_index = AssetDiscovery::asset_index(assets_dir);

        debug!("Indexed assets: {:?}", &asset_index);

        Self::load_objects(world, progress_counter, asset_index.objects);
        Self::load_maps(world, progress_counter, asset_index.maps);
    }

    /// Loads object configuration into the `World` from the specified assets directory.
    ///
    /// When this function returns, the `World` will be populated with the `CharacterAssets`
    /// resource.
    ///
    /// The normal use case for `AssetLoader` is to use the `load` function which loads both objects
    /// and maps. This method is exposed for testing the loading itself.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load the object assets into.
    /// * `progress_counter`: Tracker for loading progress.
    /// * `indexed_objects`: Index of object assets.
    pub fn load_objects(
        world: &mut World,
        progress_counter: &mut ProgressCounter,
        mut indexed_objects: HashMap<ObjectType, Vec<AssetRecord>>,
    ) {
        ObjectType::iter()
            .filter_map(|object_type| indexed_objects.remove_entry(&object_type))
            .for_each(|(object_type, asset_records)| {
                // asset_records is the list of records for one object type
                match object_type {
                    ObjectType::Character => {
                        let character_assets = asset_records
                            .into_iter()
                            .filter_map(|asset_record| {
                                debug!(
                                    "Loading `{}` from: `{}`",
                                    asset_record.asset_slug,
                                    asset_record.path.display()
                                );

                                let sprites_definition = load_in::<SpritesDefinition, _>(
                                    &asset_record.path,
                                    "sprites.toml",
                                    Format::Toml,
                                    None,
                                )
                                .expect("Failed to load sprites_definition.");

                                // TODO: <https://gitlab.com/azriel91/autexousious/issues/94>
                                let sprite_sheet_handles = {
                                    let loader = &world.read_resource::<Loader>();
                                    let texture_assets =
                                        &world.read_resource::<AssetStorage<Texture>>();
                                    let sprite_sheet_assets =
                                        &world.read_resource::<AssetStorage<SpriteSheet>>();

                                    SpriteLoader::load(
                                        progress_counter,
                                        loader,
                                        texture_assets,
                                        sprite_sheet_assets,
                                        &sprites_definition,
                                        &asset_record.path,
                                    )
                                    .expect("Failed to load textures and sprite sheets.")
                                };

                                let load_result = CharacterLoader::load(
                                    world,
                                    &asset_record.path,
                                    sprite_sheet_handles,
                                );

                                if let Err(e) = load_result {
                                    error!("Failed to load character. Reason: \n\n```\n{}\n```", e);

                                    None
                                } else {
                                    Some((asset_record.asset_slug, load_result.unwrap()))
                                }
                            })
                            .collect::<CharacterAssets>();

                        debug!("Loaded character assets: `{:?}`", character_assets);

                        world.add_resource(character_assets);
                    }
                };
            });
    }

    /// Loads map configuration into the `World` from the specified assets directory.
    ///
    /// When this function returns, the `World` will be populated with the `MapAssets` resource.
    ///
    /// The normal use case for `AssetLoader` is to use the `load` function which loads both objects
    /// and maps. This method is exposed for testing the loading itself.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load the map assets into.
    /// * `progress_counter`: Tracker for loading progress.
    /// * `indexed_maps`: Index of map assets.
    pub fn load_maps(
        world: &mut World,
        progress_counter: &mut ProgressCounter,
        indexed_maps: Vec<AssetRecord>,
    ) {
        let mut map_assets = indexed_maps
            .into_iter()
            .filter_map(|asset_record| {
                let load_result = MapLoader::load(world, &asset_record.path);

                if let Err(e) = load_result {
                    error!("Failed to load map. Reason: \n\n```\n{}\n```", e);

                    None
                } else {
                    Some((asset_record.asset_slug, load_result.unwrap()))
                }
            })
            .collect::<MapAssets>();

        let map_handle: MapHandle = {
            let loader = world.read_resource::<Loader>();
            loader.load_from_data(MAP_BLANK.clone(), progress_counter, &world.read_resource())
        };

        map_assets.insert(MAP_BLANK_SLUG.clone(), map_handle);

        debug!("Loaded map assets: `{:?}`", map_assets);

        world.add_resource(map_assets);
    }
}
