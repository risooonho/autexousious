use game_input::ControllerInput;
use object_model::{
    config::object::{CharacterSequenceId, SequenceState},
    entity::{CharacterStatus, Kinematics, ObjectStatus, ObjectStatusUpdate, RunCounter},
};

#[derive(Debug)]
pub(crate) struct SwitchSequenceOnEnd(
    /// The sequence to switch to.
    pub CharacterSequenceId,
);

impl SwitchSequenceOnEnd {
    pub fn update(
        &self,
        _controller_input: &ControllerInput,
        _character_status: &CharacterStatus,
        object_status: &ObjectStatus<CharacterSequenceId>,
        _kinematics: &Kinematics<f32>,
        _run_counter: RunCounter,
    ) -> ObjectStatusUpdate<CharacterSequenceId> {
        let mut object_status_update = ObjectStatusUpdate::default();
        if object_status.sequence_state == SequenceState::End {
            object_status_update.sequence_id = Some(self.0);
            object_status_update.sequence_state = Some(SequenceState::Begin);
        }

        object_status_update
    }
}

#[cfg(test)]
mod test {
    use game_input::ControllerInput;
    use object_model::{
        config::object::{CharacterSequenceId, SequenceState},
        entity::{CharacterStatus, Kinematics, ObjectStatus, ObjectStatusUpdate, RunCounter},
    };

    use super::SwitchSequenceOnEnd;

    #[test]
    fn no_update_when_sequence_not_ended() {
        let input = ControllerInput::new(0., 0., false, false, false, false);

        assert_eq!(
            ObjectStatusUpdate::default(),
            SwitchSequenceOnEnd(CharacterSequenceId::Stand).update(
                &input,
                &CharacterStatus::default(),
                &ObjectStatus {
                    sequence_id: CharacterSequenceId::Flinch0,
                    ..Default::default()
                },
                &Kinematics::default(),
                RunCounter::default()
            )
        );
    }

    #[test]
    fn reverts_to_stand_when_sequence_ended() {
        let input = ControllerInput::new(0., 0., false, false, false, false);

        assert_eq!(
            ObjectStatusUpdate {
                sequence_id: Some(CharacterSequenceId::Stand),
                sequence_state: Some(SequenceState::Begin),
                ..Default::default()
            },
            SwitchSequenceOnEnd(CharacterSequenceId::Stand).update(
                &input,
                &CharacterStatus::default(),
                &ObjectStatus {
                    sequence_id: CharacterSequenceId::Flinch0,
                    sequence_state: SequenceState::End,
                    ..Default::default()
                },
                &Kinematics::default(),
                RunCounter::default()
            )
        );
    }
}
