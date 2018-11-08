use std::collections::HashMap;
use std::hash::Hash;

use amethyst::{
    animation::{Animation, InterpolationFunction, Sampler},
    assets::Loader,
    prelude::*,
};
use animation_support::{ActiveHandleChannel, ActiveHandlePrimitive};
use collision_model::{
    animation::{BodyAnimationHandle, BodyFrameActiveHandle, BodyFramePrimitive},
    config::BodyFrame,
};

use BodyAnimationFrame;
use BodyAnimationSequence;

/// Loads `Animation`s from character sequences.
#[derive(Debug)]
pub struct BodyAnimationLoader;

impl BodyAnimationLoader {
    /// Loads `BodyFrame` animations and returns a hash map of their handles by sequence ID.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load animations into.
    /// * `sequences`: Sequences of the animation.
    pub fn load_into_map<'seq, SequenceId, Sequence, Frame>(
        world: &'seq World,
        sequences: &HashMap<SequenceId, Sequence>,
    ) -> HashMap<SequenceId, BodyAnimationHandle>
    where
        SequenceId: Copy + Eq + Hash + 'seq,
        Frame: BodyAnimationFrame,
        Sequence: BodyAnimationSequence<Frame = Frame> + 'seq,
    {
        Self::load(world, sequences.iter())
            .map(|(id, handle)| (*id, handle))
            .collect::<HashMap<SequenceId, BodyAnimationHandle>>()
    }

    /// Loads `BodyFrame` animations and returns a vector of their handles in order.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load animations into.
    /// * `sequences`: Sequences of the animation.
    pub fn load_into_vec<'seq, Sequences, Sequence, Frame>(
        world: &'seq World,
        sequences: Sequences,
    ) -> Vec<BodyAnimationHandle>
    where
        Sequences: Iterator<Item = &'seq Sequence>,
        Frame: BodyAnimationFrame,
        Sequence: BodyAnimationSequence<Frame = Frame> + 'seq,
    {
        sequences
            .map(|sequence| Self::sequence_to_animation(world, sequence))
            .map(|animation| {
                let loader = world.read_resource::<Loader>();
                loader.load_from_data(animation, (), &world.read_resource())
            })
            .collect::<Vec<BodyAnimationHandle>>()
    }

    /// Loads `BodyFrame` animations and returns an iterator to their handles by sequence ID.
    ///
    /// The caller is responsible for collecting the elements into the desired collection type.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load animations into.
    /// * `sequences`: Sequences of the animation.
    pub fn load<'seq, Sequences, SequenceId, Sequence, Frame>(
        world: &'seq World,
        sequences: Sequences,
    ) -> impl Iterator<Item = (&'seq SequenceId, BodyAnimationHandle)>
    where
        Sequences: Iterator<Item = (&'seq SequenceId, &'seq Sequence)>,
        SequenceId: 'seq,
        Frame: BodyAnimationFrame,
        Sequence: BodyAnimationSequence<Frame = Frame> + 'seq,
    {
        sequences
            .map(move |(id, sequence)| (id, Self::sequence_to_animation(world, sequence)))
            .map(move |(id, animation)| {
                let loader = world.read_resource::<Loader>();
                let animation_handle = loader.load_from_data(animation, (), &world.read_resource());
                (id, animation_handle)
            })
    }

    /// Maps a `Sequence` into an Amethyst `Animation`.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to store the `Animation`s.
    /// * `sequence`: `Sequence` to create the animation from.
    fn sequence_to_animation<Sequence: BodyAnimationSequence<Frame = F>, F: BodyAnimationFrame>(
        world: &World,
        sequence: &Sequence,
    ) -> Animation<BodyFrameActiveHandle> {
        let frames = sequence.frames();
        let mut input = Vec::with_capacity(frames.len() + 1);
        let mut tick_counter = 0.;
        for frame in frames {
            input.push(tick_counter);
            tick_counter += 1. + frame.wait() as f32;
        }
        input.push(tick_counter);

        let frame_sampler = Self::frame_sampler(world, sequence, input.clone());

        let loader = world.read_resource::<Loader>();
        let frame_sampler_handle = loader.load_from_data(frame_sampler, (), &world.read_resource());

        Animation {
            nodes: vec![(0, ActiveHandleChannel::Handle, frame_sampler_handle)],
        }
    }

    fn frame_sampler<Sequence: BodyAnimationSequence<Frame = F>, F: BodyAnimationFrame>(
        world: &World,
        sequence: &Sequence,
        input: Vec<f32>,
    ) -> Sampler<BodyFramePrimitive> {
        let frame_count = sequence.frames().len();
        let mut output = Vec::with_capacity(frame_count);
        sequence.frames().iter().for_each(|frame| {
            // TODO: load `BodyFrame`s and pass `Handle`s in.
            let loader = world.read_resource::<Loader>();
            let store = world.read_resource();
            let handle = loader.load_from_data(
                BodyFrame::new(frame.body().map(Clone::clone), frame.wait()),
                (),
                &store,
            );

            output.push(ActiveHandlePrimitive::Handle(handle));
        });
        let final_key_frame = output.last().cloned();
        if final_key_frame.is_some() {
            output.push(final_key_frame.unwrap());
        }

        Sampler {
            input,
            output,
            function: InterpolationFunction::Step,
        }
    }
}

#[cfg(test)]
mod test {
    use shape_model::Volume;
    use std::collections::HashMap;

    use amethyst::{
        animation::{Animation, AnimationBundle, InterpolationFunction, Sampler},
        assets::AssetStorage,
        prelude::*,
    };
    use amethyst_test::prelude::*;
    use animation_support::ActiveHandleChannel;
    use collision_model::{
        animation::{BodyAnimationHandle, BodyFrameActiveHandle, BodyFramePrimitive},
        config::{BodyFrame, Interaction},
    };

    use super::BodyAnimationLoader;
    use BodyAnimationSequence;
    use CollisionLoadingBundle;

    #[test]
    fn loads_body_animations_into_map() {
        let effect = |world: &mut World| {
            let test_sequences = test_sequences();
            let animation_handles = BodyAnimationLoader::load_into_map(world, &test_sequences);
            world.add_resource(EffectReturn(animation_handles));
        }; // kcov-ignore
        let assertion = |world: &mut World| {
            let animation_handles = &world
                .read_resource::<EffectReturn<HashMap<TestSequenceId, BodyAnimationHandle>>>()
                .0;

            // Verify animation is loaded
            verify_animation_handle(world, animation_handles.get(&TestSequenceId::Boo));
        };

        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::render_base("loads_body_animations_into_map", false)
                .with_bundle(AnimationBundle::<u32, BodyFrameActiveHandle>::new(
                    "body_frame_acs",
                    "body_frame_sis",
                ))
                .with_bundle(CollisionLoadingBundle::new())
                .with_effect(effect)
                .with_assertion(assertion)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn loads_body_animations_into_vec() {
        let effect = |world: &mut World| {
            let test_sequences = test_sequences();
            let animation_handles =
                BodyAnimationLoader::load_into_vec(world, test_sequences.values());
            world.add_resource(EffectReturn(animation_handles));
        };
        let assertion = |world: &mut World| {
            let animation_handles = &world
                .read_resource::<EffectReturn<Vec<BodyAnimationHandle>>>()
                .0;

            // Verify animation is loaded
            verify_animation_handle(world, animation_handles.first());
        };

        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::render_base("loads_body_animations_into_vec", false)
                .with_bundle(AnimationBundle::<u32, BodyFrameActiveHandle>::new(
                    "body_frame_acs",
                    "body_frame_sis",
                ))
                .with_bundle(CollisionLoadingBundle::new())
                .with_effect(effect)
                .with_assertion(assertion)
                .run()
                .is_ok()
        );
    }

    fn verify_animation_handle(world: &World, animation_handle: Option<&BodyAnimationHandle>) {
        assert!(animation_handle.is_some());

        let animation_handle = animation_handle.unwrap();
        let animation_store =
            world.read_resource::<AssetStorage<Animation<BodyFrameActiveHandle>>>();
        let animation = animation_store.get(animation_handle);
        assert!(animation.is_some());

        let animation = animation.unwrap();
        assert_eq!(1, animation.nodes.len());

        let node_0 = &animation.nodes[0];
        assert_eq!(0, node_0.0);
        assert_eq!(ActiveHandleChannel::Handle, node_0.1);

        // Verify animation samplers
        let frame_sampler_handle = &node_0.2;
        let frame_sampler_store =
            world.read_resource::<AssetStorage<Sampler<BodyFramePrimitive>>>();
        let frame_sampler = frame_sampler_store.get(frame_sampler_handle);
        assert!(frame_sampler.is_some());

        let frame_sampler = frame_sampler.unwrap();
        assert_eq!(vec![0.0, 1.0, 4.0, 6.0], frame_sampler.input);
        // TODO: Verify on handles
        // assert_eq!(
        //     vec![
        //         ActiveHandlePrimitive::Handle(10.into()),
        //         ActiveHandlePrimitive::Handle(11.into()),
        //         ActiveHandlePrimitive::Handle(12.into()),
        //         ActiveHandlePrimitive::Handle(12.into()),
        //     ],
        //     frame_sampler.output
        // );
        assert_eq!(InterpolationFunction::Step, frame_sampler.function);
    }

    fn test_sequences() -> HashMap<TestSequenceId, TestSequence> {
        // Sheet, Sprite, Wait
        let frames = vec![
            BodyFrame::new(test_body(), 0), // TU: 0 to 1
            BodyFrame::new(test_body(), 2), // TU: 1 to 4
            BodyFrame::new(test_body(), 1), // TU: 4 to 6
        ];
        let sequence = TestSequence::new(frames);
        let mut sequences = HashMap::new();
        sequences.insert(TestSequenceId::Boo, sequence);
        sequences
    }

    fn test_body() -> Option<Vec<Volume>> {
        Some(vec![Volume::Sphere {
            x: 1,
            y: 2,
            z: 3,
            r: 4,
        }])
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum TestSequenceId {
        Boo,
    }
    impl Default for TestSequenceId {
        fn default() -> Self {
            TestSequenceId::Boo
        }
    }
    #[derive(Debug, new)]
    struct TestSequence {
        frames: Vec<BodyFrame>,
    }
    impl BodyAnimationSequence for TestSequence {
        type Frame = BodyFrame;
        fn frames(&self) -> &[BodyFrame] {
            &self.frames
        }
    }
}