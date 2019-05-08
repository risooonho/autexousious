use amethyst::{core::bundle::SystemBundle, ecs::DispatcherBuilder, Error};
use application_menu::MenuItemWidgetInputSystem;
use derive_new::new;
use game_mode_selection_model::GameModeIndex;
use typename::TypeName;

use crate::GameModeSelectionWidgetUiSystem;

/// Adds the systems that set up and manage the `GameModeSelectionUi`.
///
/// The `GameInputBundle` must be added before this bundle.
#[derive(Debug, new)]
pub struct GameModeSelectionUiBundle;

impl GameModeSelectionUiBundle {
    /// Returns the system names added by this bundle.
    ///
    /// This allows consumers to specify the systems as dependencies.
    pub fn system_names() -> Vec<String> {
        vec![
            MenuItemWidgetInputSystem::<GameModeIndex>::type_name(),
            GameModeSelectionWidgetUiSystem::type_name(),
        ]
    }
}

impl<'a, 'b> SystemBundle<'a, 'b> for GameModeSelectionUiBundle {
    fn build(self, builder: &mut DispatcherBuilder<'a, 'b>) -> Result<(), Error> {
        builder.add(
            MenuItemWidgetInputSystem::<GameModeIndex>::new(),
            &MenuItemWidgetInputSystem::<GameModeIndex>::type_name(),
            &[],
        ); // kcov-ignore
        builder.add(
            GameModeSelectionWidgetUiSystem::new(),
            &GameModeSelectionWidgetUiSystem::type_name(),
            &[&MenuItemWidgetInputSystem::<GameModeIndex>::type_name()],
        ); // kcov-ignore

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use amethyst::Error;
    use amethyst_test::AmethystApplication;

    use super::GameModeSelectionUiBundle;

    #[test]
    fn bundle_build_should_succeed() -> Result<(), Error> {
        env::set_var("APP_DIR", env!("CARGO_MANIFEST_DIR"));

        AmethystApplication::ui_base::<String, String>()
            .with_bundle(GameModeSelectionUiBundle::new())
            .run()
    }
}
