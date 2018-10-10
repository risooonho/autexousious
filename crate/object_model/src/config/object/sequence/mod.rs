//! Configuration types for object sequences.
//!
//! A sequence is an independent grouping of frames, which contains not only animation information,
//! but also the collision zones, interaction, and effects.
//!
//! Sequences are shared by different object types, and are genericized by the sequence ID. This is
//! because different object types have different valid sequence IDs, and we want to be able to
//! define this at compile time rather than needing to process this at run time.

pub use self::frame::Frame;
pub use self::sequence_id::SequenceId;
pub use self::sequence_state::SequenceState;

use sprite_loading::AnimationSequence;

mod frame;
mod sequence_id;
mod sequence_state;

/// Represents an independent action sequence of an object.
///
/// This carries the information necessary for an `Animation`, as well as the effects and
/// interactions that happen during each frame of that animation.
#[derive(Clone, Debug, Deserialize, PartialEq, new)]
pub struct Sequence<SeqId: SequenceId> {
    /// ID of the sequence to switch to after this one has completed.
    ///
    /// Note: This may not be immediately after the last frame of the sequence. For example, a
    /// character that is in mid-air should remain in the last frame until it lands on the ground.
    pub next: Option<SeqId>,
    /// Key frames in the animation sequence.
    pub frames: Vec<Frame>,
}

impl<SeqId: SequenceId> AnimationSequence for Sequence<SeqId> {
    type Frame = Frame;
    fn frames(&self) -> &[Frame] {
        &self.frames
    }
}

#[cfg(test)]
mod test {
    use shape_model::{Axis, Volume};
    use toml;

    use super::{Frame, Sequence, SequenceId};

    const SEQUENCE_WITH_FRAMES: &str = r#"
        next = "Boo"
        frames = [
          { sheet = 0, sprite = 4, wait = 2 },
          { sheet = 0, sprite = 5, wait = 2 },
          { sheet = 1, sprite = 6, wait = 1 },
          { sheet = 1, sprite = 7, wait = 1 },
          { sheet = 0, sprite = 6, wait = 2 },
          { sheet = 0, sprite = 5, wait = 2 },
        ]
    "#;
    const SEQUENCE_WITH_FRAMES_EMPTY: &str = r#"
        frames = []
    "#;
    const FRAME_WITH_BODY_ALL_SPECIFIED: &str = r#"
        [[frames]]
        sheet = 0
        sprite = 0
        wait = 0
        body = [
          { box = { x = -1, y = -2, z = -3, w = 11, h = 12, d = 13 } },
          { cylinder = { axis = "x", center = -4, r = 14, l = 24 } },
          { cylinder = { axis = "y", center = -5, r = 15, l = 25 } },
          { cylinder = { axis = "z", center = -6, r = 16, l = 26 } },
          { sphere = { x = -7, y = -8, z = -9, r = 17 } },
        ]
    "#;
    const FRAME_WITH_BODY_MINIMUM_SPECIFIED: &str = r#"
        [[frames]]
        sheet = 0
        sprite = 0
        wait = 0
        body = [
          { box = { x = -1, y = -2, w = 11, h = 12 } },
          { sphere = { x = -7, y = -8, r = 17 } },
        ]
    "#;

    #[test]
    fn sequence_with_empty_frames_list_deserializes_successfully() {
        let sequence = toml::from_str::<Sequence<TestSeqId>>(SEQUENCE_WITH_FRAMES_EMPTY)
            .expect("Failed to deserialize sequence.");

        let expected = Sequence::new(None, vec![]);
        assert_eq!(expected, sequence);
    }

    #[test]
    fn sequence_with_frames() {
        let sequence = toml::from_str::<Sequence<TestSeqId>>(SEQUENCE_WITH_FRAMES)
            .expect("Failed to deserialize sequence.");

        let frames = vec![
            Frame::new(0, 4, 2, None),
            Frame::new(0, 5, 2, None),
            Frame::new(1, 6, 1, None),
            Frame::new(1, 7, 1, None),
            Frame::new(0, 6, 2, None),
            Frame::new(0, 5, 2, None),
        ];
        let expected = Sequence::new(Some(TestSeqId::Boo), frames);
        assert_eq!(expected, sequence);
    }

    #[test]
    fn sequence_with_body_specify_all_fields() {
        let sequence = toml::from_str::<Sequence<TestSeqId>>(FRAME_WITH_BODY_ALL_SPECIFIED)
            .expect("Failed to deserialize sequence.");

        let body_volumes = vec![
            Volume::Box {
                x: -1,
                y: -2,
                z: -3,
                w: 11,
                h: 12,
                d: 13,
            },
            Volume::Cylinder {
                axis: Axis::X,
                center: -4,
                r: 14,
                l: 24,
            },
            Volume::Cylinder {
                axis: Axis::Y,
                center: -5,
                r: 15,
                l: 25,
            },
            Volume::Cylinder {
                axis: Axis::Z,
                center: -6,
                r: 16,
                l: 26,
            },
            Volume::Sphere {
                x: -7,
                y: -8,
                z: -9,
                r: 17,
            },
        ];
        let frames = vec![Frame::new(0, 0, 0, Some(body_volumes))];
        let expected = Sequence::new(None, frames);
        assert_eq!(expected, sequence);
    }

    #[test]
    fn sequence_with_body_specify_minimum_fields() {
        let sequence = toml::from_str::<Sequence<TestSeqId>>(FRAME_WITH_BODY_MINIMUM_SPECIFIED)
            .expect("Failed to deserialize sequence.");

        let body_volumes = vec![
            Volume::Box {
                x: -1,
                y: -2,
                z: 0,
                w: 11,
                h: 12,
                d: 26,
            },
            Volume::Sphere {
                x: -7,
                y: -8,
                z: 0,
                r: 17,
            },
        ];
        let frames = vec![Frame::new(0, 0, 0, Some(body_volumes))];
        let expected = Sequence::new(None, frames);
        assert_eq!(expected, sequence);
    }

    #[derive(Clone, Copy, Debug, Derivative, Deserialize, PartialEq, Eq, Hash)]
    #[derivative(Default)]
    enum TestSeqId {
        #[derivative(Default)]
        Boo,
    }
    impl SequenceId for TestSeqId {}
}
