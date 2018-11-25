use game_input::ControllerInput;
use object_model::{
    config::object::CharacterSequenceId,
    entity::{CharacterStatus, Kinematics, ObjectStatus, ObjectStatusUpdate, RunCounter},
};

use character::sequence_handler::{CharacterSequenceHandler, SwitchSequenceOnLand};

const FALL_FORWARD_DESCEND_BOUNCE: SwitchSequenceOnLand =
    SwitchSequenceOnLand(CharacterSequenceId::FallForwardLand);
const FALL_FORWARD_DESCEND_LIE: SwitchSequenceOnLand =
    SwitchSequenceOnLand(CharacterSequenceId::LieFaceDown);

#[derive(Debug)]
pub(crate) struct FallForwardDescend;

impl CharacterSequenceHandler for FallForwardDescend {
    fn update(
        controller_input: &ControllerInput,
        character_status: &CharacterStatus,
        object_status: &ObjectStatus<CharacterSequenceId>,
        kinematics: &Kinematics<f32>,
        run_counter: RunCounter,
    ) -> ObjectStatusUpdate<CharacterSequenceId> {
        if kinematics.velocity[1] <= -10. {
            FALL_FORWARD_DESCEND_BOUNCE.update(
                controller_input,
                character_status,
                object_status,
                kinematics,
                run_counter,
            )
        } else {
            FALL_FORWARD_DESCEND_LIE.update(
                controller_input,
                character_status,
                object_status,
                kinematics,
                run_counter,
            )
        }
    }
}
