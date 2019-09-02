use amethyst::{
    ecs::{Read, ReadStorage, System, World, WriteStorage},
    shred::{ResourceId, SystemData},
    shrev::{EventChannel, ReaderId},
};
use derivative::Derivative;
use derive_new::new;
use map_model::play::{
    MapBoundaryEvent, MapBoundaryEventData, MapUnboundedDelete, OutOfBoundsDeleteClock,
};
use typename_derive::TypeName;

/// Number of ticks an entity has to remain out of bounds before it is deleted.
const OUT_OF_BOUNDS_DELETE_DELAY: usize = 180;

/// Adds/removes `OutOfBoundsDeleteClock`s to `MapUnboundedDelete` entities.
#[derive(Debug, Default, TypeName, new)]
pub struct MapOutOfBoundsClockAugmentSystem {
    /// Reader ID for the `MapBoundaryEvent` channel.
    #[new(default)]
    map_boundary_event_rid: Option<ReaderId<MapBoundaryEvent>>,
}

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct MapOutOfBoundsClockAugmentSystemData<'s> {
    /// `MapBoundaryEvent` channel.
    #[derivative(Debug = "ignore")]
    pub map_boundary_ec: Read<'s, EventChannel<MapBoundaryEvent>>,
    /// `MapUnboundedDelete` components.
    #[derivative(Debug = "ignore")]
    pub map_unbounded_deletes: ReadStorage<'s, MapUnboundedDelete>,
    /// `OutOfBoundsDeleteClock` components.
    #[derivative(Debug = "ignore")]
    pub out_of_bounds_delete_clocks: WriteStorage<'s, OutOfBoundsDeleteClock>,
}

impl<'s> System<'s> for MapOutOfBoundsClockAugmentSystem {
    type SystemData = MapOutOfBoundsClockAugmentSystemData<'s>;

    fn run(
        &mut self,
        MapOutOfBoundsClockAugmentSystemData {
            map_boundary_ec,
            map_unbounded_deletes,
            mut out_of_bounds_delete_clocks,
        }: Self::SystemData,
    ) {
        let map_boundary_event_rid = self
            .map_boundary_event_rid
            .as_mut()
            .expect("Expected `map_boundary_event_rid` field to be set.");

        map_boundary_ec
            .read(map_boundary_event_rid)
            .for_each(|ev| match ev {
                MapBoundaryEvent::Exit(MapBoundaryEventData { entity, .. }) => {
                    let entity = *entity;
                    if map_unbounded_deletes.contains(entity) {
                        out_of_bounds_delete_clocks
                            .insert(
                                entity,
                                OutOfBoundsDeleteClock::new(OUT_OF_BOUNDS_DELETE_DELAY),
                            )
                            .expect("Failed to insert `OutOfBoundsDeleteClock` component.");
                    }
                }
                MapBoundaryEvent::Enter(MapBoundaryEventData { entity, .. }) => {
                    let entity = *entity;
                    if map_unbounded_deletes.contains(entity)
                        && out_of_bounds_delete_clocks.contains(entity)
                    {
                        out_of_bounds_delete_clocks.remove(entity);
                    }
                }
            });
    }

    fn setup(&mut self, world: &mut World) {
        Self::SystemData::setup(world);

        self.map_boundary_event_rid = Some(
            world
                .fetch_mut::<EventChannel<MapBoundaryEvent>>()
                .register_reader(),
        );
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{
        ecs::{Builder, Entity, WorldExt},
        shrev::EventChannel,
        Error,
    };
    use amethyst_test::AmethystApplication;
    use enumflags2::BitFlags;
    use map_model::play::{
        BoundaryFace, MapBoundaryEvent, MapBoundaryEventData, MapUnboundedDelete,
        OutOfBoundsDeleteClock,
    };
    use typename::TypeName;

    use super::{MapOutOfBoundsClockAugmentSystem, OUT_OF_BOUNDS_DELETE_DELAY};

    #[test]
    fn does_not_change_out_of_bounds_delete_clock_when_no_map_boundary_event() -> Result<(), Error>
    {
        let out_of_bounds_delete_clock = OutOfBoundsDeleteClock::new_with_value(10, 5);
        run_test(
            SetupParams {
                out_of_bounds_delete_clock: Some(out_of_bounds_delete_clock),
                map_boundary_event_fn: None,
            },
            ExpectedParams {
                out_of_bounds_delete_clock: Some(out_of_bounds_delete_clock),
            },
        )
    }

    #[test]
    fn augments_out_of_bounds_delete_clock_on_exit_event() -> Result<(), Error> {
        let out_of_bounds_delete_clock = OutOfBoundsDeleteClock::new(OUT_OF_BOUNDS_DELETE_DELAY);
        run_test(
            SetupParams {
                out_of_bounds_delete_clock: None,
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Left);
                    MapBoundaryEvent::Exit(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                out_of_bounds_delete_clock: Some(out_of_bounds_delete_clock),
            },
        )
    }

    #[test]
    fn removes_out_of_bounds_delete_clock_on_enter_event() -> Result<(), Error> {
        let out_of_bounds_delete_clock = OutOfBoundsDeleteClock::new_with_value(10, 5);
        run_test(
            SetupParams {
                out_of_bounds_delete_clock: Some(out_of_bounds_delete_clock),
                map_boundary_event_fn: Some(|entity| {
                    let boundary_faces = BitFlags::from(BoundaryFace::Left);
                    MapBoundaryEvent::Enter(MapBoundaryEventData {
                        entity,
                        boundary_faces,
                    })
                }),
            },
            ExpectedParams {
                out_of_bounds_delete_clock: None,
            },
        )
    }

    fn run_test(
        SetupParams {
            out_of_bounds_delete_clock,
            map_boundary_event_fn,
        }: SetupParams,
        ExpectedParams {
            out_of_bounds_delete_clock: out_of_bounds_delete_clock_expected,
        }: ExpectedParams,
    ) -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(
                MapOutOfBoundsClockAugmentSystem::new(),
                MapOutOfBoundsClockAugmentSystem::type_name(),
                &[],
            ) // kcov-ignore
            .with_effect(move |world| {
                let entity = {
                    let mut entity_builder = world.create_entity().with(MapUnboundedDelete);

                    if let Some(out_of_bounds_delete_clock) = out_of_bounds_delete_clock {
                        entity_builder = entity_builder.with(out_of_bounds_delete_clock);
                    }

                    entity_builder.build()
                };

                if let Some(map_boundary_event_fn) = map_boundary_event_fn {
                    let map_boundary_event = map_boundary_event_fn(entity);
                    let mut map_boundary_ec =
                        world.write_resource::<EventChannel<MapBoundaryEvent>>();

                    map_boundary_ec.single_write(map_boundary_event);
                }

                world.insert(entity);
            })
            .with_assertion(move |world| {
                let entity = *world.read_resource::<Entity>();
                let out_of_bounds_delete_clocks = world.read_storage::<OutOfBoundsDeleteClock>();
                let out_of_bounds_delete_clock_actual =
                    out_of_bounds_delete_clocks.get(entity).copied();

                assert_eq!(
                    out_of_bounds_delete_clock_expected,
                    out_of_bounds_delete_clock_actual
                );
            })
            .run()
    }

    struct SetupParams {
        out_of_bounds_delete_clock: Option<OutOfBoundsDeleteClock>,
        map_boundary_event_fn: Option<fn(Entity) -> MapBoundaryEvent>,
    }

    struct ExpectedParams {
        out_of_bounds_delete_clock: Option<OutOfBoundsDeleteClock>,
    }
}
