use std::fmt::Debug;
use std::path::Path;

use amethyst;
use amethyst::prelude::*;
use game_model::config::index_configuration;
use object_model::ObjectType;
use object_model::loaded;

use object_loader::ObjectLoader;

/// `State` where resource loading takes place.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct State<'p, T: amethyst::State + 'static> {
    /// Path to the assets directory.
    assets_dir: &'p Path,
    /// The `State` that follows this one.
    #[derivative(Debug(bound = "T: Debug"))]
    next_state: Option<Box<T>>,
}

impl<'p, T: amethyst::State + 'static> State<'p, T> {
    /// Returns a new `State`
    pub fn new(assets_dir: &'p Path, next_state: Box<T>) -> Self {
        State {
            assets_dir,
            next_state: Some(next_state),
        }
    }
}

impl<'p, T: amethyst::State + 'static> amethyst::State for State<'p, T> {
    fn on_start(&mut self, world: &mut World) {
        // TODO: Start thread to load resources / assets.

        let configuration_index = index_configuration(&self.assets_dir);
        debug!("Indexed configuration: {:?}", &configuration_index);

        let mut object_loader = ObjectLoader::new(world);

        ObjectType::variants()
            .iter()
            .filter_map(|object_type| configuration_index.objects.get(object_type))
            .map(|config_records| {
                // config_records is the list of records for one object type
                config_records
                    .iter()
                    .filter_map(|config_record| object_loader.load_object(config_record).ok())
                    .collect::<Vec<loaded::Object>>()
            })
            .collect::<Vec<Vec<loaded::Object>>>();
    }

    fn fixed_update(&mut self, _world: &mut World) -> Trans {
        // TODO: Check state of resource / asset loading.
        // If it has loaded then `Trans::Switch`. Otherwise `Trans::None`

        Trans::Switch(
            self.next_state
                .take()
                .expect("Expected `next_state` to be set"),
        )
    }
}
