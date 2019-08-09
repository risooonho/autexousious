use std::convert::TryInto;

use amethyst::ecs::{Entities, Join, ReadStorage, System, WriteStorage};
use charge_model::{
    config::ChargePoints,
    play::{ChargeDelayClock, ChargeStatus, ChargeTrackerClock},
};
use derivative::Derivative;
use derive_new::new;
use shred_derive::SystemData;
use typename_derive::TypeName;

/// Ticks `ChargeTrackerClock` while `Charging`.
#[derive(Debug, Default, TypeName, new)]
pub struct ChargeIncrementSystem;

#[derive(Derivative, SystemData)]
#[derivative(Debug)]
pub struct ChargeIncrementSystemData<'s> {
    /// `Entities` resource.
    #[derivative(Debug = "ignore")]
    pub entities: Entities<'s>,
    /// `ChargeStatus` components.
    #[derivative(Debug = "ignore")]
    pub charge_statuses: ReadStorage<'s, ChargeStatus>,
    /// `ChargeDelayClock` components.
    #[derivative(Debug = "ignore")]
    pub charge_delay_clocks: WriteStorage<'s, ChargeDelayClock>,
    /// `ChargeTrackerClock` components.
    #[derivative(Debug = "ignore")]
    pub charge_tracker_clocks: WriteStorage<'s, ChargeTrackerClock>,
    /// `ChargePoints` components.
    #[derivative(Debug = "ignore")]
    pub charge_pointses: WriteStorage<'s, ChargePoints>,
}

impl<'s> System<'s> for ChargeIncrementSystem {
    type SystemData = ChargeIncrementSystemData<'s>;

    fn run(
        &mut self,
        ChargeIncrementSystemData {
            entities,
            charge_statuses,
            mut charge_delay_clocks,
            mut charge_tracker_clocks,
            mut charge_pointses,
        }: Self::SystemData,
    ) {
        (
            &entities,
            &charge_statuses,
            &mut charge_delay_clocks,
            &mut charge_tracker_clocks,
        )
            .join()
            .for_each(
                |(entity, charge_status, charge_delay_clock, charge_tracker_clock)| {
                    if *charge_status == ChargeStatus::Charging {
                        charge_delay_clock.tick();

                        if charge_delay_clock.is_complete() {
                            charge_delay_clock.reset();

                            charge_tracker_clock.tick();

                            let charge_points = ChargePoints::new(
                                (*charge_tracker_clock)
                                    .value
                                    .try_into()
                                    .expect("Failed to convert charge points `usize` to `u32`."),
                            );
                            charge_pointses
                                .insert(entity, charge_points)
                                .expect("Failed to insert `ChargePoints` component.");
                        }
                    }
                },
            );
    }
}

#[cfg(test)]
mod tests {
    use amethyst::{
        ecs::{Builder, Entity, ReadStorage},
        Error,
    };
    use amethyst_test::AmethystApplication;
    use charge_model::{
        config::ChargePoints,
        play::{ChargeDelayClock, ChargeStatus, ChargeTrackerClock},
    };

    use super::ChargeIncrementSystem;

    #[test]
    fn ticks_delay_clock_when_charging() -> Result<(), Error> {
        let charge_delay_clock = ChargeDelayClock::new(10);
        let charge_tracker_clock = ChargeTrackerClock::new(10);
        let charge_status = ChargeStatus::Charging;

        run_test(
            SetupParams {
                charge_status,
                charge_delay_clock,
                charge_tracker_clock,
            },
            |charge_delay_clock, charge_tracker_clock, charge_points| {
                let mut charge_delay_clock_expected = ChargeDelayClock::new(10);
                (*charge_delay_clock_expected).value = 1;

                let charge_tracker_clock_expected = ChargeTrackerClock::new(10);
                let charge_points_expected = ChargePoints::new(0);

                assert_eq!(Some(charge_delay_clock_expected), charge_delay_clock);
                assert_eq!(Some(charge_tracker_clock_expected), charge_tracker_clock);
                assert_eq!(Some(charge_points_expected), charge_points);
            },
        )
    }

    #[test]
    fn ticks_tracker_clock_when_delay_clock_is_complete() -> Result<(), Error> {
        let mut charge_delay_clock = ChargeDelayClock::new(10);
        (*charge_delay_clock).value = 9;
        let charge_tracker_clock = ChargeTrackerClock::new(10);
        let charge_status = ChargeStatus::Charging;

        run_test(
            SetupParams {
                charge_status,
                charge_delay_clock,
                charge_tracker_clock,
            },
            |charge_delay_clock, charge_tracker_clock, charge_points| {
                let charge_delay_clock_expected = ChargeDelayClock::new(10);

                let mut charge_tracker_clock_expected = ChargeTrackerClock::new(10);
                (*charge_tracker_clock_expected).value = 1;
                let charge_points_expected = ChargePoints::new(1);

                assert_eq!(Some(charge_delay_clock_expected), charge_delay_clock);
                assert_eq!(Some(charge_tracker_clock_expected), charge_tracker_clock);
                assert_eq!(Some(charge_points_expected), charge_points);
            },
        )
    }

    #[test]
    fn does_not_tick_clocks_when_not_charging() -> Result<(), Error> {
        let mut charge_delay_clock = ChargeDelayClock::new(10);
        (*charge_delay_clock).value = 9;
        let charge_tracker_clock = ChargeTrackerClock::new(10);
        let charge_status = ChargeStatus::BeginDelay;

        run_test(
            SetupParams {
                charge_status,
                charge_delay_clock,
                charge_tracker_clock,
            },
            |charge_delay_clock, charge_tracker_clock, charge_points| {
                let mut charge_delay_clock_expected = ChargeDelayClock::new(10);
                (*charge_delay_clock_expected).value = 9;

                let charge_tracker_clock_expected = ChargeTrackerClock::new(10);
                let charge_points_expected = ChargePoints::new(0);

                assert_eq!(Some(charge_delay_clock_expected), charge_delay_clock);
                assert_eq!(Some(charge_tracker_clock_expected), charge_tracker_clock);
                assert_eq!(Some(charge_points_expected), charge_points);
            },
        )
    }

    fn run_test(
        SetupParams {
            charge_status,
            charge_delay_clock,
            charge_tracker_clock,
        }: SetupParams,
        assertion_fn: fn(
            Option<ChargeDelayClock>,
            Option<ChargeTrackerClock>,
            Option<ChargePoints>,
        ),
    ) -> Result<(), Error> {
        AmethystApplication::blank()
            .with_system(ChargeIncrementSystem::new(), "", &[])
            .with_setup(move |world| {
                let entity = world
                    .create_entity()
                    .with(charge_status)
                    .with(charge_delay_clock)
                    .with(charge_tracker_clock)
                    .with(ChargePoints::new(0))
                    .build();

                world.add_resource(entity);
            })
            .with_assertion(move |world| {
                let entity = *world.read_resource::<Entity>();
                let (charge_delay_clocks, charge_tracker_clocks, charge_pointses) = world
                    .system_data::<(
                        ReadStorage<'_, ChargeDelayClock>,
                        ReadStorage<'_, ChargeTrackerClock>,
                        ReadStorage<'_, ChargePoints>,
                    )>();

                let charge_delay_clock = charge_delay_clocks.get(entity).copied();
                let charge_tracker_clock = charge_tracker_clocks.get(entity).copied();
                let charge_points = charge_pointses.get(entity).copied();

                assertion_fn(charge_delay_clock, charge_tracker_clock, charge_points);
            })
            .run()
    }

    struct SetupParams {
        charge_status: ChargeStatus,
        charge_delay_clock: ChargeDelayClock,
        charge_tracker_clock: ChargeTrackerClock,
    }
}
