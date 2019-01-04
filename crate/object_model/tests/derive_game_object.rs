use std::collections::HashMap;

use amethyst::{
    assets::AssetLoaderSystemData,
    ecs::{storage::VecStorage, Component},
    Result,
};
use amethyst_test::AmethystApplication;
use derivative::Derivative;
use object_model::{
    config::object::SequenceId,
    game_object,
    loaded::{GameObject, Object, SequenceEndTransition, SequenceEndTransitions},
};
use specs_derive::Component;

#[derive(Clone, Component, Copy, Debug, Derivative, PartialEq, Eq, Hash)]
#[derivative(Default)]
#[storage(VecStorage)]
enum TestSequenceId {
    #[derivative(Default)]
    Boo,
}
impl SequenceId for TestSequenceId {}

#[game_object(TestSequenceId)]
#[derive(Debug)]
struct MagicObject;

#[test]
fn game_object_attribute_generates_handle_and_transitions_fields() -> Result<()> {
    AmethystApplication::blank()
        .with_assertion(|world| {
            let sequence_end_transitions = {
                let mut transitions = SequenceEndTransitions::default();
                transitions
                    .0
                    .insert(TestSequenceId::Boo, SequenceEndTransition::default());
                transitions
            };
            let object_handle = {
                let object = Object::new(Vec::new(), HashMap::new());
                world.exec(
                    |asset_loader: AssetLoaderSystemData<Object<TestSequenceId>>| {
                        asset_loader.load_from_data(object, ())
                    },
                )
            };

            let magic_object = MagicObject {
                object_handle: object_handle.clone(),
                sequence_end_transitions: sequence_end_transitions.clone(),
            };

            assert_eq!(&object_handle, magic_object.object_handle());
            assert_eq!(
                &sequence_end_transitions,
                magic_object.sequence_end_transitions()
            );
        })
        .run()
}
