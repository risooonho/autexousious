use amethyst::{ecs::prelude::*, shrev::EventChannel};
use asset_model::loaded::SlugAndHandle;
use character_selection_model::{CharacterSelection, CharacterSelectionEvent};
use derivative::Derivative;
use derive_new::new;
use game_input::InputControlled;
use game_input_model::{
    Axis, AxisEventData, ControlAction, ControlActionEventData, ControlInputEvent,
};
use game_model::loaded::CharacterAssets;
use log::debug;
use shred_derive::SystemData;

use typename_derive::TypeName;

use crate::{CharacterSelectionWidget, WidgetState};

/// System that processes controller input and generates `CharacterSelectionEvent`s.
///
/// This is not private because consumers may use `CharacterSelectionWidgetInputSystem::type_name()` to
/// specify this as a dependency of another system.
#[derive(Debug, Default, TypeName, new)]
pub(crate) struct CharacterSelectionWidgetInputSystem {
    /// Reader ID for the `ControlInputEvent` channel.
    #[new(default)]
    control_input_event_rid: Option<ReaderId<ControlInputEvent>>,
}

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub(crate) struct CharacterSelectionWidgetInputResources<'s> {
    /// `CharacterSelectionWidget` components.
    #[derivative(Debug = "ignore")]
    pub character_selection_widgets: WriteStorage<'s, CharacterSelectionWidget>,
    /// `InputControlled` components.
    #[derivative(Debug = "ignore")]
    pub input_controlleds: ReadStorage<'s, InputControlled>,
    /// `Character` assets.
    #[derivative(Debug = "ignore")]
    pub character_assets: Read<'s, CharacterAssets>,
    /// `CharacterSelectionEvent` channel.
    #[derivative(Debug = "ignore")]
    pub character_selection_ec: Write<'s, EventChannel<CharacterSelectionEvent>>,
}

type CharacterSelectionWidgetInputSystemData<'s> = (
    Read<'s, EventChannel<ControlInputEvent>>,
    CharacterSelectionWidgetInputResources<'s>,
);

impl CharacterSelectionWidgetInputSystem {
    fn select_previous_character(
        character_assets: &CharacterAssets,
        widget: &mut CharacterSelectionWidget,
    ) {
        let (first_character_slug, first_character_handle) = character_assets
            .iter()
            .next()
            .expect("Expected at least one character to be loaded.");
        let (last_character_slug, last_character_handle) = character_assets
            .iter()
            .next_back()
            .expect("Expected at least one character to be loaded.");
        widget.selection = match widget.selection {
            CharacterSelection::Id(SlugAndHandle {
                slug: ref character_slug,
                ..
            }) => {
                if character_slug == first_character_slug {
                    CharacterSelection::Random(
                        (first_character_slug, first_character_handle).into(),
                    )
                } else {
                    let next_character = character_assets
                        .iter()
                        .rev()
                        .skip_while(|(slug, _handle)| slug != &character_slug)
                        .nth(1); // skip current selection

                    if let Some(next_character) = next_character {
                        CharacterSelection::Id(next_character.into())
                    } else {
                        CharacterSelection::Random(
                            (first_character_slug, first_character_handle).into(),
                        )
                    }
                }
            }
            CharacterSelection::Random(..) => {
                CharacterSelection::Id((last_character_slug, last_character_handle).into())
            }
        };
    }

    fn select_next_character(
        character_assets: &CharacterAssets,
        widget: &mut CharacterSelectionWidget,
    ) {
        let (first_character_slug, first_character_handle) = character_assets
            .iter()
            .next()
            .expect("Expected at least one character to be loaded.");
        let last_character_slug = character_assets
            .keys()
            .next_back()
            .expect("Expected at least one character to be loaded.");
        widget.selection = match widget.selection {
            CharacterSelection::Id(SlugAndHandle {
                slug: ref character_slug,
                ..
            }) => {
                if character_slug == last_character_slug {
                    CharacterSelection::Random(
                        (first_character_slug, first_character_handle).into(),
                    )
                } else {
                    let next_character = character_assets
                        .iter()
                        .skip_while(|(slug, _handle)| slug != &character_slug)
                        .nth(1); // skip current selection

                    if let Some(next_character) = next_character {
                        CharacterSelection::Id(next_character.into())
                    } else {
                        CharacterSelection::Random(
                            (first_character_slug, first_character_handle).into(),
                        )
                    }
                }
            }
            CharacterSelection::Random(..) => {
                CharacterSelection::Id((first_character_slug, first_character_handle).into())
            }
        };
    }

    fn handle_event(
        CharacterSelectionWidgetInputResources {
            ref mut character_selection_widgets,
            ref input_controlleds,
            ref character_assets,
            ref mut character_selection_ec,
        }: &mut CharacterSelectionWidgetInputResources,
        event: ControlInputEvent,
    ) {
        match event {
            ControlInputEvent::Axis(axis_event_data) => {
                if let Some(character_selection_widget) =
                    character_selection_widgets.get_mut(axis_event_data.entity)
                {
                    Self::handle_axis_event(
                        &character_assets,
                        character_selection_widget,
                        axis_event_data,
                    )
                }
            }
            ControlInputEvent::ControlAction(control_action_event_data) => {
                if let (Some(character_selection_widget), Some(input_controlled)) = (
                    character_selection_widgets.get_mut(control_action_event_data.entity),
                    input_controlleds.get(control_action_event_data.entity),
                ) {
                    Self::handle_control_action_event(
                        character_selection_ec,
                        character_selection_widget,
                        *input_controlled,
                        control_action_event_data,
                    )
                }
            }
        }
    }

    fn handle_axis_event(
        character_assets: &CharacterAssets,
        character_selection_widget: &mut CharacterSelectionWidget,
        axis_event_data: AxisEventData,
    ) {
        match (character_selection_widget.state, axis_event_data.axis) {
            (WidgetState::CharacterSelect, Axis::X) if axis_event_data.value < 0. => {
                Self::select_previous_character(character_assets, character_selection_widget);
            }
            (WidgetState::CharacterSelect, Axis::X) if axis_event_data.value > 0. => {
                Self::select_next_character(character_assets, character_selection_widget);
            }
            _ => {}
        }
    }

    fn handle_control_action_event(
        character_selection_ec: &mut EventChannel<CharacterSelectionEvent>,
        character_selection_widget: &mut CharacterSelectionWidget,
        input_controlled: InputControlled,
        control_action_event_data: ControlActionEventData,
    ) {
        match (
            character_selection_widget.state,
            control_action_event_data.control_action,
            control_action_event_data.value,
        ) {
            (WidgetState::Inactive, ControlAction::Attack, true) => {
                debug!("Controller {} active.", input_controlled.controller_id);
                character_selection_widget.state = WidgetState::CharacterSelect;
            }
            (WidgetState::CharacterSelect, ControlAction::Jump, true) => {
                debug!("Controller {} inactive.", input_controlled.controller_id);
                character_selection_widget.state = WidgetState::Inactive;
            }
            (WidgetState::CharacterSelect, ControlAction::Attack, true) => {
                debug!("Controller {} ready.", input_controlled.controller_id);
                character_selection_widget.state = WidgetState::Ready;

                // Send character selection event
                let character_selection_event = CharacterSelectionEvent::Select {
                    controller_id: input_controlled.controller_id,
                    character_selection: character_selection_widget.selection.clone(),
                };
                debug!(
                    "Sending character selection event: {:?}",
                    &character_selection_event // kcov-ignore
                );
                character_selection_ec.single_write(character_selection_event);
            }
            (WidgetState::Ready, ControlAction::Jump, true) => {
                character_selection_widget.state = WidgetState::CharacterSelect;

                let character_selection_event = CharacterSelectionEvent::Deselect {
                    controller_id: input_controlled.controller_id,
                };
                debug!(
                    "Sending character selection event: {:?}",
                    &character_selection_event // kcov-ignore
                );
                character_selection_ec.single_write(character_selection_event);
            }
            (WidgetState::Ready, ControlAction::Attack, true) => {
                let character_selection_event = CharacterSelectionEvent::Confirm;
                debug!(
                    "Sending character selection event: {:?}",
                    &character_selection_event // kcov-ignore
                );
                character_selection_ec.single_write(character_selection_event);
            }
            _ => {}
        }
    }
}

impl<'s> System<'s> for CharacterSelectionWidgetInputSystem {
    type SystemData = CharacterSelectionWidgetInputSystemData<'s>;

    fn run(
        &mut self,
        (control_input_ec, mut character_selection_widget_input_resources): Self::SystemData,
    ) {
        let control_input_event_rid = self
            .control_input_event_rid
            .as_mut()
            .expect("Expected `control_input_event_rid` field to be set.");

        control_input_ec
            .read(control_input_event_rid)
            .for_each(|ev| {
                Self::handle_event(&mut character_selection_widget_input_resources, *ev);
            });
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.control_input_event_rid = Some(
            res.fetch_mut::<EventChannel<ControlInputEvent>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod test {
    use amethyst::{
        assets::Prefab,
        ecs::{Builder, Entity, SystemData, World},
        shrev::{EventChannel, ReaderId},
        Error,
    };
    use application_test_support::AutexousiousApplication;
    use asset_model::{config::AssetSlug, loaded::SlugAndHandle};
    use assets_test::ASSETS_CHAR_BAT_SLUG;
    use character_loading::CharacterPrefab;
    use character_selection_model::{CharacterSelection, CharacterSelectionEvent};
    use game_input::InputControlled;
    use game_input_model::{
        Axis, AxisEventData, ControlAction, ControlActionEventData, ControlInputEvent,
    };
    use game_model::loaded::CharacterAssets;
    use typename::TypeName;

    use super::{CharacterSelectionWidgetInputSystem, CharacterSelectionWidgetInputSystemData};
    use crate::{CharacterSelectionWidget, WidgetState};

    #[test]
    fn does_not_send_event_when_controller_input_empty() -> Result<(), Error> {
        run_test(
            "updates_widget_character_select_to_ready_and_sends_event_when_input_attack",
            SetupParams {
                widget_state: WidgetState::Inactive,
                character_selection_fn: character_selection_random,
                control_input_event_fn: None,
            },
            ExpectedParams {
                widget_state: WidgetState::Inactive,
                character_selection_fn: character_selection_random,
                character_selection_events_fn: empty_events,
            },
        )
    }

    #[test]
    fn updates_widget_inactive_to_character_select_when_input_attack() -> Result<(), Error> {
        run_test(
            "updates_widget_inactive_to_character_select_when_input_attack",
            SetupParams {
                widget_state: WidgetState::Inactive,
                character_selection_fn: character_selection_random,
                control_input_event_fn: Some(press_attack),
            },
            ExpectedParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: character_selection_random,
                character_selection_events_fn: empty_events,
            },
        )
    }

    #[test]
    fn updates_widget_character_select_to_ready_and_sends_event_when_input_attack(
    ) -> Result<(), Error> {
        run_test(
            "updates_widget_character_select_to_ready_and_sends_event_when_input_attack",
            SetupParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                control_input_event_fn: Some(press_attack),
            },
            ExpectedParams {
                widget_state: WidgetState::Ready,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                character_selection_events_fn: |world| {
                    let bat_snh = SlugAndHandle::from((&*world, ASSETS_CHAR_BAT_SLUG.clone()));
                    vec![CharacterSelectionEvent::Select {
                        controller_id: 123,
                        character_selection: CharacterSelection::Id(bat_snh),
                    }]
                },
            },
        )
    }

    #[test]
    fn sends_confirm_event_when_widget_ready_and_input_attack() -> Result<(), Error> {
        run_test(
            "sends_confirm_event_when_widget_ready_and_input_attack",
            SetupParams {
                widget_state: WidgetState::Ready,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                control_input_event_fn: Some(press_attack),
            },
            ExpectedParams {
                widget_state: WidgetState::Ready,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                character_selection_events_fn: |_world| vec![CharacterSelectionEvent::Confirm],
            },
        )
    }

    #[test]
    fn selects_last_character_when_input_left_and_selection_random() -> Result<(), Error> {
        run_test(
            "selects_last_character_when_input_left_and_selection_random",
            SetupParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: character_selection_random,
                control_input_event_fn: Some(press_left),
            },
            ExpectedParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: |world| {
                    let last_char = last_character(world);
                    CharacterSelection::Id(last_char)
                },
                character_selection_events_fn: |world| {
                    let last_char = last_character(world);
                    vec![CharacterSelectionEvent::Select {
                        controller_id: 123,
                        character_selection: CharacterSelection::Id(last_char),
                    }]
                },
            },
        )
    }

    #[test]
    fn selects_first_character_when_input_right_and_selection_random() -> Result<(), Error> {
        run_test(
            "selects_random_when_input_right_and_selection_last_character",
            SetupParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: character_selection_random,
                control_input_event_fn: Some(press_right),
            },
            ExpectedParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: |world| {
                    let first_char = first_character(world);
                    CharacterSelection::Id(first_char)
                },
                character_selection_events_fn: |world| {
                    let first_char = first_character(world);
                    vec![CharacterSelectionEvent::Select {
                        controller_id: 123,
                        character_selection: CharacterSelection::Id(first_char),
                    }]
                },
            },
        )
    }

    #[test]
    fn selects_random_when_input_right_and_selection_last_character() -> Result<(), Error> {
        run_test(
            "selects_random_when_input_right_and_selection_last_character",
            SetupParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                control_input_event_fn: Some(press_right),
            },
            ExpectedParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: character_selection_random,
                character_selection_events_fn: |world| {
                    let first_char = first_character(world);
                    vec![CharacterSelectionEvent::Select {
                        controller_id: 123,
                        character_selection: CharacterSelection::Id(first_char),
                    }]
                },
            },
        )
    }

    #[test]
    fn updates_widget_ready_to_character_select_and_sends_event_when_input_jump(
    ) -> Result<(), Error> {
        run_test(
            "updates_widget_ready_to_character_select_and_sends_event_when_input_jump",
            SetupParams {
                widget_state: WidgetState::Ready,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                control_input_event_fn: Some(press_jump),
            },
            ExpectedParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                character_selection_events_fn: |_world| {
                    vec![CharacterSelectionEvent::Deselect { controller_id: 123 }]
                },
            },
        )
    }

    #[test]
    fn updates_widget_character_select_to_inactive_when_input_jump() -> Result<(), Error> {
        run_test(
            "updates_widget_character_select_to_inactive_when_input_jump",
            SetupParams {
                widget_state: WidgetState::CharacterSelect,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                control_input_event_fn: Some(press_jump),
            },
            ExpectedParams {
                widget_state: WidgetState::Inactive,
                character_selection_fn: |world| {
                    character_selection_id(world, ASSETS_CHAR_BAT_SLUG.clone())
                },
                character_selection_events_fn: empty_events,
            },
        )
    }

    fn run_test(
        test_name: &str,
        SetupParams {
            widget_state: setup_widget_state,
            character_selection_fn: setup_character_selection_fn,
            control_input_event_fn,
        }: SetupParams,
        ExpectedParams {
            widget_state: expected_widget_state,
            character_selection_fn: expected_character_selection_fn,
            character_selection_events_fn,
        }: ExpectedParams,
    ) -> Result<(), Error> {
        AutexousiousApplication::config_base(test_name, false)
            .with_system(
                CharacterSelectionWidgetInputSystem::new(),
                CharacterSelectionWidgetInputSystem::type_name(),
                &[],
            ) // kcov-ignore
            .with_setup(move |world| {
                CharacterSelectionWidgetInputSystemData::setup(&mut world.res);

                let setup_character_selection = setup_character_selection_fn(world);
                let entity = widget_entity(world, setup_widget_state, setup_character_selection);
                world.add_resource(entity);

                let event_channel_reader = world
                    .write_resource::<EventChannel<CharacterSelectionEvent>>()
                    .register_reader(); // kcov-ignore

                world.add_resource(event_channel_reader);
            })
            .with_effect(move |world| {
                if let Some(control_input_event_fn) = control_input_event_fn {
                    let entity = *world.read_resource::<Entity>();
                    world
                        .write_resource::<EventChannel<ControlInputEvent>>()
                        .single_write(control_input_event_fn(entity));
                }
            })
            .with_assertion(move |world| {
                let expected_character_selection = expected_character_selection_fn(world);
                assert_widget(
                    world,
                    CharacterSelectionWidget::new(
                        expected_widget_state,
                        expected_character_selection,
                    ),
                )
            })
            .with_assertion(move |world| {
                let character_selection_events = character_selection_events_fn(world);
                assert_events(world, character_selection_events);
            })
            .run()
    }

    fn character_selection_id(world: &mut World, slug: AssetSlug) -> CharacterSelection {
        let snh = SlugAndHandle::from((&*world, slug.clone()));
        CharacterSelection::Id(snh)
    }

    fn character_selection_random(world: &mut World) -> CharacterSelection {
        let first_char = first_character(world);
        CharacterSelection::Random(first_char)
    }

    fn press_left(entity: Entity) -> ControlInputEvent {
        ControlInputEvent::Axis(AxisEventData {
            entity,
            axis: Axis::X,
            value: -1.,
        })
    }

    fn press_right(entity: Entity) -> ControlInputEvent {
        ControlInputEvent::Axis(AxisEventData {
            entity,
            axis: Axis::X,
            value: 1.,
        })
    }

    fn press_jump(entity: Entity) -> ControlInputEvent {
        ControlInputEvent::ControlAction(ControlActionEventData {
            entity,
            control_action: ControlAction::Jump,
            value: true,
        })
    }

    fn press_attack(entity: Entity) -> ControlInputEvent {
        ControlInputEvent::ControlAction(ControlActionEventData {
            entity,
            control_action: ControlAction::Attack,
            value: true,
        })
    }

    fn empty_events(_world: &mut World) -> Vec<CharacterSelectionEvent> {
        vec![]
    }

    fn first_character(world: &mut World) -> SlugAndHandle<Prefab<CharacterPrefab>> {
        world
            .read_resource::<CharacterAssets>()
            .iter()
            .next()
            .expect("Expected at least one character to be loaded.")
            .into()
    }

    fn last_character(world: &mut World) -> SlugAndHandle<Prefab<CharacterPrefab>> {
        world
            .read_resource::<CharacterAssets>()
            .iter()
            .next_back()
            .expect("Expected at least one character to be loaded.")
            .into()
    }

    fn widget_entity(
        world: &mut World,
        widget_state: WidgetState,
        character_selection: CharacterSelection,
    ) -> Entity {
        world
            .create_entity()
            .with(CharacterSelectionWidget::new(
                widget_state,
                character_selection,
            ))
            .with(InputControlled::new(123))
            .build()
    }

    fn assert_widget(world: &mut World, expected: CharacterSelectionWidget) {
        let widget_entity = world.read_resource::<Entity>();

        let widgets = world.read_storage::<CharacterSelectionWidget>();
        let widget = widgets
            .get(*widget_entity)
            .expect("Expected entity to have `CharacterSelectionWidget` component.");

        assert_eq!(expected, *widget);
    }

    fn assert_events(world: &mut World, events: Vec<CharacterSelectionEvent>) {
        let mut event_channel_reader =
            &mut world.write_resource::<ReaderId<CharacterSelectionEvent>>();

        let character_selection_event_channel =
            world.read_resource::<EventChannel<CharacterSelectionEvent>>();
        let character_selection_event_iter =
            character_selection_event_channel.read(&mut event_channel_reader);

        let expected_events_iter = events.into_iter();
        expected_events_iter
            .zip(character_selection_event_iter)
            .for_each(|(expected_event, actual)| assert_eq!(expected_event, *actual));
    }

    struct SetupParams {
        widget_state: WidgetState,
        character_selection_fn: fn(&mut World) -> CharacterSelection,
        control_input_event_fn: Option<fn(Entity) -> ControlInputEvent>,
    }

    struct ExpectedParams {
        widget_state: WidgetState,
        character_selection_fn: fn(&mut World) -> CharacterSelection,
        character_selection_events_fn: fn(&mut World) -> Vec<CharacterSelectionEvent>,
    }
}
