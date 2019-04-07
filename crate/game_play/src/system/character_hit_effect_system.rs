use amethyst::{
    ecs::{Read, Resources, System, SystemData, WriteStorage},
    shrev::{EventChannel, ReaderId},
};
use character_model::config::CharacterSequenceId;
use collision_model::{
    config::{Hit, Interaction, InteractionKind},
    play::HitEvent,
};
use derive_new::new;
use object_model::entity::HealthPoints;
use sequence_model::entity::SequenceStatus;
use typename_derive::TypeName;

/// Determines collision effects for characters.
#[derive(Debug, Default, TypeName, new)]
pub(crate) struct CharacterHitEffectSystem {
    /// Reader ID for the `HitEvent` event channel.
    #[new(default)]
    hit_event_rid: Option<ReaderId<HitEvent>>,
}

type CharacterHitEffectSystemData<'s> = (
    Read<'s, EventChannel<HitEvent>>,
    WriteStorage<'s, HealthPoints>,
    WriteStorage<'s, CharacterSequenceId>,
    WriteStorage<'s, SequenceStatus>,
);

impl<'s> System<'s> for CharacterHitEffectSystem {
    type SystemData = CharacterHitEffectSystemData<'s>;

    fn run(
        &mut self,
        (hit_ec, mut health_pointses, mut character_sequence_ids, mut sequence_statuses): Self::SystemData,
    ) {
        // Read from channel
        hit_ec
            .read(
                self.hit_event_rid
                    .as_mut()
                    .expect("Expected reader ID to exist for CharacterHitEffectSystem."),
            )
            .for_each(|ev| {
                // Fetch health points of the object that is hit.
                let health_points = health_pointses.get_mut(ev.to);
                let character_sequence_id = character_sequence_ids.get_mut(ev.to);
                let sequence_status = sequence_statuses.get_mut(ev.to);

                if let (Some(health_points), Some(character_sequence_id), Some(sequence_status)) =
                    (health_points, character_sequence_id, sequence_status)
                {
                    // TODO: Select damage sequence based on balance / fall points.
                    // TODO: Split this system with health check system.
                    let Interaction {
                        kind: InteractionKind::Hit(Hit { hp_damage, .. }),
                        ..
                    } = ev.interaction;
                    if health_points.0 < hp_damage {
                        *health_points = HealthPoints(0);
                    } else {
                        (*health_points) -= hp_damage;
                    }

                    let next_sequence_id = if *health_points == 0 {
                        CharacterSequenceId::FallForwardAscend
                    } else if *character_sequence_id == CharacterSequenceId::Flinch0 {
                        CharacterSequenceId::Flinch1
                    } else {
                        CharacterSequenceId::Flinch0
                    };

                    // Set sequence id
                    *character_sequence_id = next_sequence_id;
                    *sequence_status = SequenceStatus::Begin;
                }
            });
    }

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);
        self.hit_event_rid = Some(res.fetch_mut::<EventChannel<HitEvent>>().register_reader());
    }
}