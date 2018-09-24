use std::collections::HashMap;
use std::hash::Hash;

use amethyst::{
    animation::{
        Animation, InterpolationFunction, Sampler, SpriteRenderChannel, SpriteRenderPrimitive,
    },
    assets::{Handle, Loader},
    prelude::*,
    renderer::SpriteRender,
};

use AnimationFrame;
use AnimationSequence;

/// Loads `Animation`s from character sequences.
#[derive(Debug)]
pub struct SpriteRenderAnimationLoader;

impl SpriteRenderAnimationLoader {
    /// Loads `SpriteRender` animations and returns a hash map of their handles by sequence ID.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load animations into.
    /// * `sequences`: Sequences of the animation.
    /// * `sprite_sheet_index_offset`: Offset of the sprite sheet IDs in the `SpriteSheetSet`.
    pub fn load_into_map<'seq, SequenceId, Sequence, Frame>(
        world: &'seq World,
        sequences: &HashMap<SequenceId, Sequence>,
        sprite_sheet_index_offset: u64,
    ) -> HashMap<SequenceId, Handle<Animation<SpriteRender>>>
    where
        SequenceId: Copy + Eq + Hash + 'seq,
        Frame: AnimationFrame,
        Sequence: AnimationSequence<Frame = Frame> + 'seq,
    {
        Self::load(world, sequences.iter(), sprite_sheet_index_offset)
            .map(|(id, handle)| (*id, handle))
            .collect::<HashMap<SequenceId, Handle<Animation<SpriteRender>>>>()
    }

    /// Loads `SpriteRender` animations and returns a vector of their handles in order.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load animations into.
    /// * `sequences`: Sequences of the animation.
    /// * `sprite_sheet_index_offset`: Offset of the sprite sheet IDs in the `SpriteSheetSet`.
    pub fn load_into_vec<'seq, Sequences, Sequence, Frame>(
        world: &'seq World,
        sequences: Sequences,
        sprite_sheet_index_offset: u64,
    ) -> Vec<Handle<Animation<SpriteRender>>>
    where
        Sequences: Iterator<Item = &'seq Sequence>,
        Frame: AnimationFrame,
        Sequence: AnimationSequence<Frame = Frame> + 'seq,
    {
        sequences
            .map(|sequence| Self::sequence_to_animation(world, sprite_sheet_index_offset, sequence))
            .map(|animation| {
                let loader = world.read_resource::<Loader>();
                loader.load_from_data(animation, (), &world.read_resource())
            }).collect::<Vec<Handle<Animation<SpriteRender>>>>()
    }

    /// Loads `SpriteRender` animations and returns an iterator to their handles by sequence ID.
    ///
    /// The caller is responsible for collecting the elements into the desired collection type.
    ///
    /// # Parameters
    ///
    /// * `world`: `World` to load animations into.
    /// * `sequences`: Sequences of the animation.
    /// * `sprite_sheet_index_offset`: Offset of the sprite sheet IDs in the `SpriteSheetSet`.
    pub fn load<'seq, Sequences, SequenceId, Sequence, Frame>(
        world: &'seq World,
        sequences: Sequences,
        sprite_sheet_index_offset: u64,
    ) -> impl Iterator<Item = (&'seq SequenceId, Handle<Animation<SpriteRender>>)>
    where
        Sequences: Iterator<Item = (&'seq SequenceId, &'seq Sequence)>,
        SequenceId: 'seq,
        Frame: AnimationFrame,
        Sequence: AnimationSequence<Frame = Frame> + 'seq,
    {
        sequences
            .map(move |(id, sequence)| {
                (
                    id,
                    Self::sequence_to_animation(world, sprite_sheet_index_offset, sequence),
                )
            }).map(move |(id, animation)| {
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
    /// * `sprite_sheet_index_offset`: Offset of the sprite sheet IDs in the `SpriteSheetSet`.
    /// * `sequence`: `Sequence` to create the animation from.
    fn sequence_to_animation<Sequence: AnimationSequence<Frame = F>, F: AnimationFrame>(
        world: &World,
        sprite_sheet_index_offset: u64,
        sequence: &Sequence,
    ) -> Animation<SpriteRender> {
        let frames = sequence.frames();
        let mut input = Vec::with_capacity(frames.len() + 1);
        let mut tick_counter = 0.;
        for frame in frames {
            input.push(tick_counter);
            tick_counter += 1. + frame.wait() as f32;
        }
        input.push(tick_counter);

        let sprite_sheet_sampler =
            Self::sprite_sheet_sampler(sprite_sheet_index_offset, sequence, input.clone());
        let sprite_index_sampler = Self::sprite_index_sampler(sequence, input);

        let loader = world.read_resource::<Loader>();
        let sprite_sheet_sampler_handle =
            loader.load_from_data(sprite_sheet_sampler, (), &world.read_resource());
        let sprite_index_sampler_handle =
            loader.load_from_data(sprite_index_sampler, (), &world.read_resource());

        Animation {
            nodes: vec![
                (
                    0,
                    SpriteRenderChannel::SpriteSheet,
                    sprite_sheet_sampler_handle,
                ),
                (
                    0,
                    SpriteRenderChannel::SpriteIndex,
                    sprite_index_sampler_handle,
                ),
            ],
        }
    }

    fn sprite_sheet_sampler<Sequence: AnimationSequence<Frame = F>, F: AnimationFrame>(
        sprite_sheet_index_offset: u64,
        sequence: &Sequence,
        input: Vec<f32>,
    ) -> Sampler<SpriteRenderPrimitive> {
        let mut output = sequence
            .frames()
            .iter()
            .map(|frame: &F| {
                SpriteRenderPrimitive::SpriteSheet(
                    sprite_sheet_index_offset + frame.texture_index() as u64,
                )
            }).collect::<Vec<SpriteRenderPrimitive>>();
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

    fn sprite_index_sampler<Sequence: AnimationSequence<Frame = F>, F: AnimationFrame>(
        sequence: &Sequence,
        input: Vec<f32>,
    ) -> Sampler<SpriteRenderPrimitive> {
        let mut output = sequence
            .frames()
            .iter()
            .map(|frame: &F| SpriteRenderPrimitive::SpriteIndex(frame.sprite_index()))
            .collect::<Vec<SpriteRenderPrimitive>>();
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
    use std::collections::HashMap;

    use amethyst::{
        animation::{
            Animation, InterpolationFunction, Sampler, SpriteRenderChannel, SpriteRenderPrimitive,
        },
        assets::{AssetStorage, Handle},
        prelude::*,
        renderer::SpriteRender,
    };
    use amethyst_test_support::prelude::*;

    use super::SpriteRenderAnimationLoader;
    use AnimationFrame;
    use AnimationSequence;

    #[test]
    fn loads_sprite_render_animations_into_map() {
        let effect = |world: &mut World| {
            let sprite_sheet_index_offset = 10;
            let test_sequences = test_sequences();
            let animation_handles = SpriteRenderAnimationLoader::load_into_map(
                world,
                &test_sequences,
                sprite_sheet_index_offset,
            );
            world.add_resource(EffectReturn(animation_handles));
        }; // kcov-ignore
        let assertion = |world: &mut World| {
            let animation_handles = &world
                .read_resource::<EffectReturn<HashMap<TestSequenceId, Handle<Animation<SpriteRender>>>>>(
                )
                .0;

            // Verify animation is loaded
            verify_animation_handle(world, animation_handles.get(&TestSequenceId::Boo));
        };

        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::render_base("loads_sprite_render_animations_into_map", false)
                .with_effect(effect)
                .with_assertion(assertion)
                .run()
                .is_ok()
        );
    }

    #[test]
    fn loads_sprite_render_animations_into_vec() {
        let effect = |world: &mut World| {
            let sprite_sheet_index_offset = 10;
            let test_sequences = test_sequences();
            let animation_handles = SpriteRenderAnimationLoader::load_into_vec(
                world,
                test_sequences.values(),
                sprite_sheet_index_offset,
            );
            world.add_resource(EffectReturn(animation_handles));
        };
        let assertion = |world: &mut World| {
            let animation_handles = &world
                .read_resource::<EffectReturn<Vec<Handle<Animation<SpriteRender>>>>>()
                .0;

            // Verify animation is loaded
            verify_animation_handle(world, animation_handles.first());
        };

        // kcov-ignore-start
        assert!(
            // kcov-ignore-end
            AmethystApplication::render_base("loads_sprite_render_animations_into_vec", false)
                .with_effect(effect)
                .with_assertion(assertion)
                .run()
                .is_ok()
        );
    }

    fn verify_animation_handle(
        world: &World,
        animation_handle: Option<&Handle<Animation<SpriteRender>>>,
    ) {
        assert!(animation_handle.is_some());

        let animation_handle = animation_handle.unwrap();
        let animation_store = world.read_resource::<AssetStorage<Animation<SpriteRender>>>();
        let animation = animation_store.get(animation_handle);
        assert!(animation.is_some());

        let animation = animation.unwrap();
        assert_eq!(2, animation.nodes.len());

        let node_0 = &animation.nodes[0];
        assert_eq!(0, node_0.0);
        assert_eq!(SpriteRenderChannel::SpriteSheet, node_0.1);

        let node_1 = &animation.nodes[1];
        assert_eq!(0, node_1.0);
        assert_eq!(SpriteRenderChannel::SpriteIndex, node_1.1);

        // Verify animation samplers
        let sprite_sheet_sampler_handle = &node_0.2;
        let sprite_sheet_sampler_store =
            world.read_resource::<AssetStorage<Sampler<SpriteRenderPrimitive>>>();
        let sprite_sheet_sampler = sprite_sheet_sampler_store.get(sprite_sheet_sampler_handle);
        assert!(sprite_sheet_sampler.is_some());

        let sprite_sheet_sampler = sprite_sheet_sampler.unwrap();
        assert_eq!(vec![0.0, 1.0, 4.0, 6.0], sprite_sheet_sampler.input);
        assert_eq!(
            vec![
                SpriteRenderPrimitive::SpriteSheet(10),
                SpriteRenderPrimitive::SpriteSheet(11),
                SpriteRenderPrimitive::SpriteSheet(10),
                SpriteRenderPrimitive::SpriteSheet(10),
            ],
            sprite_sheet_sampler.output
        );
        assert_eq!(InterpolationFunction::Step, sprite_sheet_sampler.function);

        let sprite_index_sampler_handle = &node_1.2;
        let sprite_index_sampler_store =
            world.read_resource::<AssetStorage<Sampler<SpriteRenderPrimitive>>>();
        let sprite_index_sampler = sprite_index_sampler_store.get(sprite_index_sampler_handle);
        assert!(sprite_index_sampler.is_some());

        let sprite_index_sampler = sprite_index_sampler.unwrap();
        assert_eq!(vec![0.0, 1.0, 4.0, 6.0], sprite_index_sampler.input);
        assert_eq!(
            vec![
                SpriteRenderPrimitive::SpriteIndex(1),
                SpriteRenderPrimitive::SpriteIndex(2),
                SpriteRenderPrimitive::SpriteIndex(3),
                SpriteRenderPrimitive::SpriteIndex(3),
            ],
            sprite_index_sampler.output
        );
        assert_eq!(InterpolationFunction::Step, sprite_index_sampler.function);
    }

    fn test_sequences() -> HashMap<TestSequenceId, TestSequence> {
        // Sheet, Sprite, Wait
        let frames = vec![
            TestFrame::new(0, 1, 0), // TU: 0 to 1
            TestFrame::new(1, 2, 2), // TU: 1 to 4
            TestFrame::new(0, 3, 1), // TU: 4 to 6
        ];
        let sequence = TestSequence::new(frames);
        let mut sequences = HashMap::new();
        sequences.insert(TestSequenceId::Boo, sequence);
        sequences
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
        frames: Vec<TestFrame>,
    }
    impl AnimationSequence for TestSequence {
        type Frame = TestFrame;
        fn frames(&self) -> &[TestFrame] {
            &self.frames
        }
    }
    #[derive(Debug, new)]
    struct TestFrame {
        sheet: usize,
        sprite: usize,
        wait: u32,
    }
    impl AnimationFrame for TestFrame {
        fn texture_index(&self) -> usize {
            self.sheet
        }
        fn sprite_index(&self) -> usize {
            self.sprite
        }
        fn wait(&self) -> u32 {
            self.wait
        }
    }
}