use amethyst::{
    core::bundle::SystemBundle,
    ecs::{DispatcherBuilder, World},
    Error,
};
use application_event::AppEventVariant;
use derive_new::new;
use stdio_spi::MapperSystem;
use typename::TypeName;

use crate::CharacterSelectionEventStdinMapper;

/// Adds a `MapperSystem<CharacterSelectionEventStdinMapper>` to the `World`.
#[derive(Debug, new)]
pub struct CharacterSelectionStdioBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for CharacterSelectionStdioBundle {
    fn build(
        self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(
            MapperSystem::<CharacterSelectionEventStdinMapper>::new(
                AppEventVariant::CharacterSelection,
            ),
            &MapperSystem::<CharacterSelectionEventStdinMapper>::type_name(),
            &[],
        ); // kcov-ignore
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use amethyst::{ecs::WorldExt, shrev::EventChannel, Error};
    use amethyst_test::AmethystApplication;
    use asset_model::loaded::AssetIdMappings;
    use stdio_spi::VariantAndTokens;

    use super::CharacterSelectionStdioBundle;

    #[test]
    fn bundle_should_add_mapper_system_to_dispatcher() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_bundle(CharacterSelectionStdioBundle::new())
            // kcov-ignore-start
            .with_effect(|world| {
                world.read_resource::<EventChannel<VariantAndTokens>>();
                world.read_resource::<AssetIdMappings>();
            })
            // kcov-ignore-end
            .run()
    }
}
