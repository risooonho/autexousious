use amethyst::{
    core::bundle::{Result, SystemBundle},
    ecs::prelude::*,
};

use StdinSystem;

/// Adds the `StdinSystem` to the `World` with id `"stdin_system"`.
#[derive(Debug, new)]
pub struct StdioViewBundle;

impl<'a, 'b> SystemBundle<'a, 'b> for StdioViewBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<()> {
        builder.add(StdinSystem::new(), "stdin_system", &[]);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use amethyst::shrev::EventChannel;
    use amethyst_test_support::prelude::*;
    use application_input::ApplicationEvent;

    use super::StdioViewBundle;

    #[test]
    fn bundle_should_add_stdin_system_to_dispatcher() {
        env::set_var("APP_DIR", env!("CARGO_MANIFEST_DIR"));
        assert!(
            AmethystApplication::blank()
                .with_bundle(StdioViewBundle)
                .with_effect(|world| {
                    world.read_resource::<EventChannel<ApplicationEvent>>();
                })
                .run()
                .is_ok()
        );
    }
}
