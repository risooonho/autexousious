use std::{fmt::Debug, marker::PhantomData};

use amethyst::{ecs::prelude::*, prelude::*, shrev::EventChannel};
use application_event::AppEvent;
use application_state::{AppState, AppStateBuilder, AutexState};
use character_selection_model::{
    CharacterSelectionEntityId, CharacterSelectionEvent, CharacterSelections,
    CharacterSelectionsStatus,
};
use derivative::Derivative;
use derive_new::new;
use log::{debug, info};
use state_inventory;

/// `State` where character selection takes place.
///
/// This state is not intended to be constructed directly, but through the
/// [`CharacterSelectionStateBuilder`][state_builder].
///
/// # Type Parameters
///
/// * `F`: Function to construct the state to return after character selection is complete.
/// * `S`: State to return.
///
/// [state_builder]: character_selection_state/struct.CharacterSelectionStateBuilder.html
pub type CharacterSelectionState<'a, 'b, F, S> =
    AppState<'a, 'b, CharacterSelectionStateDelegate<'a, 'b, F, S>, CharacterSelectionEntityId>;

state_inventory::submit!(CharacterSelectionState);

/// Builder for a `CharacterSelectionState`.
///
/// `SystemBundle`s to run in the `CharacterSelectionState`'s dispatcher are registered on this
/// builder.
///
/// # Type Parameters
///
/// * `F`: Function to construct the state to return after character selection is complete.
/// * `S`: `State` to delegate to.
pub type CharacterSelectionStateBuilder<'a, 'b, F, S> = AppStateBuilder<
    'a,
    'b,
    CharacterSelectionStateDelegate<'a, 'b, F, S>,
    CharacterSelectionEntityId,
>;

/// Delegate `State` for character selection.
///
/// This state is not intended to be used directly, but wrapped in an `AppState`. The
/// `CharacterSelectionState` is an alias with this as a delegate state.
///
/// # Type Parameters
///
/// * `F`: Function to construct the state to return after character selection is complete.
/// * `S`: State to return.
#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct CharacterSelectionStateDelegate<'a, 'b, F, S>
where
    F: Fn() -> Box<S>,
    S: AutexState<'a, 'b> + 'static,
{
    /// The `State` that follows this one.
    #[derivative(Debug(bound = "F: Debug"))]
    next_state_fn: F,
    /// `PhantomData`.
    marker: PhantomData<dyn AutexState<'a, 'b>>,
}

impl<'a, 'b, F, S> CharacterSelectionStateDelegate<'a, 'b, F, S>
where
    F: Fn() -> Box<S>,
    S: AutexState<'a, 'b> + 'static,
{
    fn initialize_character_selections(&mut self, world: &mut World) {
        let mut selections_status = world.write_resource::<CharacterSelectionsStatus>();
        *selections_status = CharacterSelectionsStatus::Waiting;
    }
}

impl<'a, 'b, F, S> State<GameData<'a, 'b>, AppEvent>
    for CharacterSelectionStateDelegate<'a, 'b, F, S>
where
    F: Fn() -> Box<S>,
    S: AutexState<'a, 'b> + 'static,
{
    fn on_start(&mut self, mut data: StateData<'_, GameData<'a, 'b>>) {
        self.initialize_character_selections(&mut data.world);
    }

    fn on_resume(&mut self, data: StateData<'_, GameData<'a, 'b>>) {
        let mut selections_status = data.world.write_resource::<CharacterSelectionsStatus>();
        *selections_status = CharacterSelectionsStatus::Confirmed;
    }

    fn handle_event(
        &mut self,
        data: StateData<'_, GameData<'a, 'b>>,
        event: AppEvent,
    ) -> Trans<GameData<'a, 'b>, AppEvent> {
        if let AppEvent::CharacterSelection(character_selection_event) = event {
            debug!(
                "Received character_selection_event: {:?}",
                character_selection_event
            );
            let mut channel = data
                .world
                .write_resource::<EventChannel<CharacterSelectionEvent>>();
            channel.single_write(character_selection_event);
        }
        Trans::None
    }

    fn update(
        &mut self,
        data: StateData<'_, GameData<'a, 'b>>,
    ) -> Trans<GameData<'a, 'b>, AppEvent> {
        let selections_status = data.world.read_resource::<CharacterSelectionsStatus>();
        if *selections_status == CharacterSelectionsStatus::Ready {
            let character_selections = data.world.read_resource::<CharacterSelections>();
            info!(
                "character_selections: `{:?}`",
                &character_selections.selections
            );

            // TODO: `Trans:Push` when we have a proper character selection menu.
            Trans::Switch((self.next_state_fn)())
        } else {
            Trans::None
        }
    }
}
