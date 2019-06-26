use amethyst::{
    ecs::{Read, ReadStorage, Resources, System, SystemData, Write},
    shrev::{EventChannel, ReaderId},
};
use collision_model::play::{CollisionEvent, ContactEvent};
use derive_new::new;
use spawn_model::play::SpawnParent;
use typename_derive::TypeName;

/// Detects whether a `ContactEvent` occurs when a `CollisionEvent` happens.
///
/// This system determines if contact happens or not -- e.g. objects on the same team may or may not
/// contact each other depending on the type of `Interaction`.
#[derive(Debug, Default, TypeName, new)]
pub struct ContactDetectionSystem {
    /// Reader ID for the `CollisionEvent` event channel.
    #[new(default)]
    collision_event_rid: Option<ReaderId<CollisionEvent>>,
}

type ContactDetectionSystemData<'s> = (
    Read<'s, EventChannel<CollisionEvent>>,
    ReadStorage<'s, SpawnParent>,
    Write<'s, EventChannel<ContactEvent>>,
);

impl<'s> System<'s> for ContactDetectionSystem {
    type SystemData = ContactDetectionSystemData<'s>;

    fn run(&mut self, (collision_ec, spawn_parents, mut contact_ec): Self::SystemData) {
        let contact_events =
            collision_ec
                .read(self.collision_event_rid.as_mut().expect(
                    "Expected `collision_event_rid` to exist for `ContactDetectionSystem`.",
                ))
                .filter(|ev| {
                    // This assumes `ev.from` is the hitting object entity. If we have a separate
                    // entity for each `Interaction`, then this assumption breaks, and we need to
                    // traverse the entity hierarchy to find the object entity.
                    let entity_hitter = ev.from;

                    // This assumes `ev.to` is the hit object entity. If we have a separate
                    // entity for each `Body`, then this assumption breaks, and we need to
                    // traverse the entity hierarchy to find the object entity.
                    let entity_hit = ev.to;

                    let dont_hit_spawn_parent = spawn_parents
                        .get(entity_hitter)
                        .map(|spawn_parent| spawn_parent.entity != entity_hit)
                        .unwrap_or(true);

                    let dont_hit_spawned_object = spawn_parents
                        .get(entity_hit)
                        .map(|spawn_parent| spawn_parent.entity != entity_hitter)
                        .unwrap_or(true);

                    dont_hit_spawn_parent && dont_hit_spawned_object
                })
                .map(|ev| ContactEvent::new(ev.from, ev.to, ev.interaction.clone(), ev.body))
                .collect::<Vec<ContactEvent>>();

        contact_ec.iter_write(contact_events);
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.collision_event_rid = Some(
            res.fetch_mut::<EventChannel<CollisionEvent>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{
        ecs::{Builder, Entity, World},
        shrev::{EventChannel, ReaderId},
        Error,
    };
    use amethyst_test::AmethystApplication;
    use collision_model::{
        config::{Hit, HitLimit, HitRepeatDelay, Interaction, InteractionKind},
        play::{CollisionEvent, ContactEvent},
    };
    use object_status_model::config::StunPoints;
    use pretty_assertions::assert_eq;
    use shape_model::Volume;
    use spawn_model::play::SpawnParent;

    use super::ContactDetectionSystem;

    const HIT_LIMIT: u32 = 3;

    #[test]
    fn inserts_contact_event_when_neither_entity_is_spawn_parent() -> Result<(), Error> {
        run_test(
            SpawnParentVariant::NoSpawnParent,
            |entity_from, entity_to| vec![contact_event(entity_from, entity_to)],
        )
    }

    #[test]
    fn does_not_insert_contact_event_when_hitter_entity_is_spawn_parent() -> Result<(), Error> {
        run_test(SpawnParentVariant::HitterEntityIsSpawnParent, |_, _| vec![])
    }

    #[test]
    fn does_not_insert_contact_event_when_hit_entity_is_spawn_parent() -> Result<(), Error> {
        run_test(SpawnParentVariant::HitEntityIsSpawnParent, |_, _| vec![])
    }

    fn run_test(
        spawn_parent_variant: SpawnParentVariant,
        events_expected_fn: fn(Entity, Entity) -> Vec<ContactEvent>,
    ) -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(ContactDetectionSystem::new(), "", &[])
            .with_setup(setup_event_reader)
            .with_effect(move |world| {
                let entity_from = world.create_entity().build();
                let entity_to = world.create_entity().build();

                match spawn_parent_variant {
                    SpawnParentVariant::HitterEntityIsSpawnParent => {
                        let mut spawn_parents = world.write_storage::<SpawnParent>();
                        spawn_parents
                            .insert(entity_from, SpawnParent::new(entity_to))
                            .expect("Failed to insert `SpawnParent` component.");
                    }
                    SpawnParentVariant::HitEntityIsSpawnParent => {
                        let mut spawn_parents = world.write_storage::<SpawnParent>();
                        spawn_parents
                            .insert(entity_to, SpawnParent::new(entity_from))
                            .expect("Failed to insert `SpawnParent` component.");
                    }
                    SpawnParentVariant::NoSpawnParent => {}
                }

                send_event(world, collision_event(entity_from, entity_to));

                world.add_resource((entity_from, entity_to));
            })
            .with_assertion(move |world| {
                let (entity_from, entity_to) = *world.read_resource::<(Entity, Entity)>();
                let events_expected = events_expected_fn(entity_from, entity_to);
                assert_events(world, events_expected);
            })
            .run()
    }

    fn setup_event_reader(world: &mut World) {
        let contact_event_rid = world
            .write_resource::<EventChannel<ContactEvent>>()
            .register_reader(); // kcov-ignore

        world.add_resource(contact_event_rid);
    }

    fn send_event(world: &mut World, event: CollisionEvent) {
        let mut ec = world.write_resource::<EventChannel<CollisionEvent>>();
        ec.single_write(event)
    } // kcov-ignore

    fn collision_event(entity_from: Entity, entity_to: Entity) -> CollisionEvent {
        CollisionEvent::new(
            entity_from,
            entity_to,
            interaction(HitLimit::Limit(HIT_LIMIT)),
            body(),
        )
    }

    fn contact_event(entity_from: Entity, entity_to: Entity) -> ContactEvent {
        ContactEvent::new(
            entity_from,
            entity_to,
            interaction(HitLimit::Limit(HIT_LIMIT)),
            body(),
        )
    }

    fn interaction(hit_limit: HitLimit) -> Interaction {
        Interaction::new(
            InteractionKind::Hit(Hit::new(
                HitRepeatDelay::new(4),
                hit_limit,
                0,
                0,
                StunPoints::default(),
            )),
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

    fn assert_events(world: &mut World, contact_events_expected: Vec<ContactEvent>) {
        let contact_ec = world.read_resource::<EventChannel<ContactEvent>>();
        let mut contact_event_rid = world.write_resource::<ReaderId<ContactEvent>>();
        let contact_events = contact_ec
            .read(&mut contact_event_rid)
            .collect::<Vec<&ContactEvent>>();

        let contact_events_expected = contact_events_expected
            .iter()
            .collect::<Vec<&ContactEvent>>();
        assert_eq!(contact_events_expected, contact_events);
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum SpawnParentVariant {
        HitterEntityIsSpawnParent,
        HitEntityIsSpawnParent,
        NoSpawnParent,
    }
}
