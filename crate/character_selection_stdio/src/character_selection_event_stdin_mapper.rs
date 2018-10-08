use std::str::FromStr;

use character_selection_model::{CharacterSelection, CharacterSelectionEvent};
use game_input::ControllerId;
use game_model::{
    config::AssetSlug,
    loaded::{CharacterAssets, SlugAndHandle},
};
use stdio_spi::{Result, StdinMapper};

use CharacterSelectionEventArgs;

/// Builds a `CharacterSelectionEvent` from stdin tokens.
#[derive(Debug, TypeName)]
pub struct CharacterSelectionEventStdinMapper;

impl CharacterSelectionEventStdinMapper {
    fn map_select_event(
        character_assets: &CharacterAssets,
        controller_id: ControllerId,
        selection: &str,
    ) -> Result<CharacterSelectionEvent> {
        let character_selection = match selection {
            "random" => {
                let snh = SlugAndHandle::from(
                    character_assets
                        .iter()
                        .next()
                        .expect("Expected at least one character to be loaded."),
                );
                CharacterSelection::Random(snh)
            }
            slug_str => {
                let slug = AssetSlug::from_str(slug_str)?;
                let handle = character_assets
                    .get(&slug)
                    .ok_or_else(|| format!("No character found with asset slug `{}`.", slug))?
                    .clone();

                let snh = SlugAndHandle { slug, handle };
                CharacterSelection::Id(snh)
            }
        };

        let character_selection_event = CharacterSelectionEvent::Select {
            controller_id,
            character_selection,
        };

        Ok(character_selection_event)
    }
}

impl StdinMapper for CharacterSelectionEventStdinMapper {
    type Resource = CharacterAssets;
    type Event = CharacterSelectionEvent;
    type Args = CharacterSelectionEventArgs;

    fn map(character_assets: &CharacterAssets, args: Self::Args) -> Result<Self::Event> {
        match args {
            CharacterSelectionEventArgs::Select {
                controller_id,
                selection,
            } => Self::map_select_event(character_assets, controller_id, &selection),
            CharacterSelectionEventArgs::Deselect { controller_id } => {
                Ok(CharacterSelectionEvent::Deselect { controller_id })
            }
            CharacterSelectionEventArgs::Confirm => Ok(CharacterSelectionEvent::Confirm),
        }
    }
}

#[cfg(test)]
mod tests {
    use application_test_support::AutexousiousApplication;
    use assets_test::ASSETS_CHAR_BAT_SLUG;
    use character_selection_model::{CharacterSelection, CharacterSelectionEvent};
    use game_model::loaded::{CharacterAssets, SlugAndHandle};
    use stdio_spi::{ErrorKind, Result, StdinMapper};

    use super::CharacterSelectionEventStdinMapper;
    use CharacterSelectionEventArgs;

    #[test]
    fn returns_err_when_asset_slug_invalid() {
        let controller_id = 0;
        let selection = "invalid".to_string();
        let args = CharacterSelectionEventArgs::Select {
            controller_id,
            selection,
        };
        let character_assets = CharacterAssets::new();

        let result = CharacterSelectionEventStdinMapper::map(&character_assets, args);

        expect_err_msg(
            result,
            "Expected exactly one `/` in slug string: \"invalid\".",
        );
    }

    #[test]
    fn returns_err_when_character_does_not_exist_for_slug() {
        let controller_id = 0;
        let selection = "test/non_existent".to_string();
        let args = CharacterSelectionEventArgs::Select {
            controller_id,
            selection,
        };
        let character_assets = CharacterAssets::new();

        let result = CharacterSelectionEventStdinMapper::map(&character_assets, args);

        expect_err_msg(
            result,
            "No character found with asset slug `test/non_existent`.",
        );
    }

    #[test]
    fn maps_select_id_event() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::config_base("maps_select_id_event", false)
                .with_assertion(|world| {
                    let controller_id = 1;
                    let args = CharacterSelectionEventArgs::Select {
                        controller_id,
                        selection: ASSETS_CHAR_BAT_SLUG.to_string(),
                    };
                    let character_assets = world.read_resource::<CharacterAssets>();

                    let result = CharacterSelectionEventStdinMapper::map(&*character_assets, args);

                    assert!(result.is_ok());
                    let snh =
                        SlugAndHandle::from((&*character_assets, ASSETS_CHAR_BAT_SLUG.clone()));
                    let character_selection = CharacterSelection::Id(snh);
                    assert_eq!(
                        CharacterSelectionEvent::Select {
                            controller_id,
                            character_selection
                        },
                        result.unwrap()
                    )
                })
                .run()
                .is_ok()
        );
    }

    #[test]
    fn maps_select_random_event() {
        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AutexousiousApplication::config_base("maps_select_random_event", false)
                .with_assertion(|world| {
                    let controller_id = 1;
                    let args = CharacterSelectionEventArgs::Select {
                        controller_id,
                        selection: "random".to_string(),
                    };
                    let character_assets = world.read_resource::<CharacterAssets>();

                    let result = CharacterSelectionEventStdinMapper::map(&*character_assets, args);

                    assert!(result.is_ok());
                    let snh = SlugAndHandle::from(
                        character_assets
                            .iter()
                            .next()
                            .expect("Expected at least one character to be loaded."),
                    );
                    let character_selection = CharacterSelection::Random(snh);
                    assert_eq!(
                        CharacterSelectionEvent::Select {
                            controller_id,
                            character_selection
                        },
                        result.unwrap()
                    )
                })
                .run()
                .is_ok()
        );
    }

    #[test]
    fn maps_deselect_event() {
        let controller_id = 0;
        let args = CharacterSelectionEventArgs::Deselect { controller_id };
        let character_assets = CharacterAssets::new();

        let result = CharacterSelectionEventStdinMapper::map(&character_assets, args);

        assert!(result.is_ok());
        assert_eq!(
            CharacterSelectionEvent::Deselect { controller_id },
            result.unwrap()
        )
    }

    #[test]
    fn maps_confirm_event() {
        let args = CharacterSelectionEventArgs::Confirm;
        let character_assets = CharacterAssets::new();

        let result = CharacterSelectionEventStdinMapper::map(&character_assets, args);

        assert!(result.is_ok());
        assert_eq!(CharacterSelectionEvent::Confirm, result.unwrap())
    }

    fn expect_err_msg(result: Result<CharacterSelectionEvent>, expected: &str) {
        assert!(result.is_err());
        match result.unwrap_err().kind() {
            ErrorKind::Msg(ref s) => assert_eq!(expected, s),
            // kcov-ignore-start
            _ => panic!("Expected `ErrorKind::Msg({:?})`.", expected),
            // kcov-ignore-end
        }
    }
}
