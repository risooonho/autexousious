use amethyst::{
    ecs::{Read, Resources, System, SystemData, WriteStorage},
    shrev::{EventChannel, ReaderId},
};
use collision_model::{
    config::{Interaction, InteractionKind},
    play::HitEvent,
};
use derive_new::new;
use logic_clock::LogicClock;
use sequence_model::entity::FrameFreezeClock;

use typename_derive::TypeName;

/// Creates `FrameFreezeClock`s for new `Hit` collisions.
///
/// This attaches `FrameFreezeClock` to the entity with the `Interaction`.
#[derive(Debug, Default, TypeName, new)]
pub struct FrameFreezeClockAugmentSystem {
    /// Reader ID for the `HitEvent` event channel.
    #[new(default)]
    hit_event_rid: Option<ReaderId<HitEvent>>,
}

type FrameFreezeClockAugmentSystemData<'s> = (
    Read<'s, EventChannel<HitEvent>>,
    WriteStorage<'s, FrameFreezeClock>,
);

impl<'s> System<'s> for FrameFreezeClockAugmentSystem {
    type SystemData = FrameFreezeClockAugmentSystemData<'s>;

    fn run(&mut self, (collision_ec, mut frame_freeze_clocks): Self::SystemData) {
        // Read from channel
        collision_ec
            .read(
                self.hit_event_rid
                    .as_mut()
                    .expect("Expected reader ID to exist for FrameFreezeClockAugmentSystem."),
            )
            .for_each(|ev| {
                // Only add `FrameFreezeClock` for `Hit` interactions.
                let Interaction {
                    kind: InteractionKind::Hit(_),
                    ..
                } = ev.interaction;

                let frame_freeze_clock = FrameFreezeClock::new(LogicClock::new(2));
                frame_freeze_clocks
                    .insert(ev.from, frame_freeze_clock)
                    .expect("Failed to insert `FrameFreezeClock`.");
            });
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.hit_event_rid = Some(res.fetch_mut::<EventChannel<HitEvent>>().register_reader());
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{
        ecs::{Builder, Entity, World},
        shrev::EventChannel,
        Error,
    };
    use amethyst_test::AmethystApplication;
    use collision_model::{
        config::{Hit, HitLimit, HitRepeatDelay, Interaction, InteractionKind},
        play::HitEvent,
    };
    use logic_clock::LogicClock;
    use sequence_model::entity::FrameFreezeClock;
    use shape_model::Volume;

    use super::FrameFreezeClockAugmentSystem;

    #[test]
    fn inserts_frame_freeze_clock_for_hitter() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(FrameFreezeClockAugmentSystem::new(), "", &[])
            .with_effect(|world| {
                let entity_from = world.create_entity().build();
                let entity_to = world.create_entity().build();

                let event = HitEvent::new(entity_from, entity_to, interaction(), body());
                send_event(world, event);

                world.add_resource(entity_from);
            })
            .with_assertion(|world| {
                let entity_from = *world.read_resource::<Entity>();
                let frame_freeze_clocks = world.read_storage::<FrameFreezeClock>();
                let frame_freeze_clock = frame_freeze_clocks.get(entity_from);

                assert_eq!(
                    Some(&FrameFreezeClock::new(LogicClock::new(2))),
                    frame_freeze_clock
                );
            })
            .run()
    }

    #[test]
    fn multiple_hit_events_only_results_in_one_freeze_frame() -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(FrameFreezeClockAugmentSystem::new(), "", &[])
            .with_effect(|world| {
                let entity_from = world.create_entity().build();
                let entity_to_0 = world.create_entity().build();
                let entity_to_1 = world.create_entity().build();

                let event = HitEvent::new(entity_from, entity_to_0, interaction(), body());
                send_event(world, event);
                let event = HitEvent::new(entity_from, entity_to_1, interaction(), body());
                send_event(world, event);

                world.add_resource(entity_from);
            })
            .with_assertion(|world| {
                let entity_from = *world.read_resource::<Entity>();
                let frame_freeze_clocks = world.read_storage::<FrameFreezeClock>();
                let frame_freeze_clock = frame_freeze_clocks.get(entity_from);

                assert_eq!(
                    Some(&FrameFreezeClock::new(LogicClock::new(2))),
                    frame_freeze_clock
                );
            })
            .run()
    }

    fn send_event(world: &mut World, event: HitEvent) {
        let mut ec = world.write_resource::<EventChannel<HitEvent>>();
        ec.single_write(event)
    }

    fn interaction() -> Interaction {
        Interaction::new(
            InteractionKind::Hit(Hit::new(HitRepeatDelay::new(4), HitLimit::Unlimited, 0, 0)),
            vec![],
            true,
        )
    }

    fn body() -> Volume {
        Volume::Box {
            x: 0,
            y: 0,
            z: 0,
            w: 1,
            h: 1,
            d: 1,
        }
    }
}