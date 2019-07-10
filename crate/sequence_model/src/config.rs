//! Contains the types that represent the configuration on disk.

pub use self::{
    control_transition::ControlTransition, control_transition_multiple::ControlTransitionMultiple,
    control_transition_single::ControlTransitionSingle, control_transitions::ControlTransitions,
    sequence_end_transition::SequenceEndTransition, sequence_id::SequenceId, wait::Wait,
};

mod control_transition;
mod control_transition_multiple;
mod control_transition_single;
mod control_transitions;
mod sequence_end_transition;
mod sequence_id;
mod wait;
