use amethyst::{core::bundle::SystemBundle, ecs::DispatcherBuilder, Error};
use application_event::AppEventVariant;
use derive_new::new;
use stdio_spi::MapperSystem;
use typename::TypeName;

use crate::MapSelectionEventStdinMapper;

/// Adds a `MapperSystem<MapSelectionEventStdinMapper>` to the `World`.
#[derive(Debug, new)]
pub struct MapSelectionStdioBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for MapSelectionStdioBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            MapperSystem::<MapSelectionEventStdinMapper>::new(AppEventVariant::MapSelection),
            &MapperSystem::<MapSelectionEventStdinMapper>::type_name(),
            &[],
        ); // kcov-ignore
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use amethyst::shrev::EventChannel;
    use amethyst_test::prelude::*;
    use asset_model::loaded::MapAssets;
    use stdio_spi::VariantAndTokens;

    use super::MapSelectionStdioBundle;

    #[test]
    fn bundle_should_add_mapper_system_to_dispatcher() {
        env::set_var("APP_DIR", env!("CARGO_MANIFEST_DIR"));
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::blank()
                .with_bundle(MapSelectionStdioBundle::new())
                // kcov-ignore-start
                .with_effect(|world| {
                    world.read_resource::<EventChannel<VariantAndTokens>>();
                    world.read_resource::<MapAssets>();
                })
                // kcov-ignore-end
                .run()
                .is_ok()
        );
    }
}
