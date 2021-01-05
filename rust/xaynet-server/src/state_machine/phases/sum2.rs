use async_trait::async_trait;
use tokio::time::{timeout, Duration};
use tracing::{debug, info};

use crate::{
    state_machine::{
        phases::{Handler, Phase, PhaseName, PhaseState, PhaseStateError, Shared, Unmask},
        requests::{StateMachineRequest, Sum2Request},
        RequestError,
        StateMachine,
    },
    storage::{CoordinatorStorage, ModelStorage},
};
use xaynet_core::{
    mask::{Aggregation, MaskObject},
    SumParticipantPublicKey,
};

/// The sum2 state.
#[derive(Debug)]
pub struct Sum2 {
    /// The aggregator for masked models.
    model_agg: Aggregation,
    /// The number of sum2 messages successfully processed.
    accepted: u64,
    /// The number of sum2 messages failed to processed.
    rejected: u64,
    /// The number of sum2 messages discarded without being processed.
    discarded: u64,
}

#[async_trait]
impl<C, M> Phase<C, M> for PhaseState<Sum2, C, M>
where
    Self: Handler,
    C: CoordinatorStorage,
    M: ModelStorage,
{
    const NAME: PhaseName = PhaseName::Sum2;

    async fn run(&mut self) -> Result<(), PhaseStateError> {
        let min_time = self.shared.state.min_sum_time;
        let max_time = self.shared.state.max_sum_time;
        debug!(
            "in sum2 phase for min {} and max {} seconds",
            min_time, max_time,
        );
        self.process_during(Duration::from_secs(min_time)).await?;

        let time_left = max_time - min_time;
        timeout(Duration::from_secs(time_left), self.process_until_enough()).await??;

        info!(
            "in total {} sum2 messages accepted (min {} and max {} required)",
            self.private.accepted, self.shared.state.min_sum_count, self.shared.state.max_sum_count,
        );
        info!("in total {} sum2 messages rejected", self.private.rejected);
        info!(
            "in total {} sum2 messages discarded",
            self.private.discarded,
        );

        Ok(())
    }

    /// Moves from the sum2 state to the next state.
    ///
    /// See the [module level documentation](../index.html) for more details.
    fn next(self) -> Option<StateMachine<C, M>> {
        Some(PhaseState::<Unmask, _, _>::new(self.shared, self.private.model_agg).into())
    }
}

#[async_trait]
impl<C, M> Handler for PhaseState<Sum2, C, M>
where
    C: CoordinatorStorage,
    M: ModelStorage,
{
    async fn handle_request(&mut self, req: StateMachineRequest) -> Result<(), RequestError> {
        if let StateMachineRequest::Sum2(Sum2Request {
            participant_pk,
            model_mask,
        }) = req
        {
            self.update_mask_dict(participant_pk, model_mask).await
        } else {
            Err(RequestError::MessageRejected)
        }
    }

    fn has_enough_messages(&self) -> bool {
        self.private.accepted >= self.shared.state.min_sum_count
    }

    fn has_overmuch_messages(&self) -> bool {
        self.private.accepted >= self.shared.state.max_sum_count
    }

    fn increment_accepted(&mut self) {
        self.private.accepted += 1;
        debug!(
            "{} sum2 messages accepted (min {} and max {} required)",
            self.private.accepted, self.shared.state.min_sum_count, self.shared.state.max_sum_count,
        );
    }

    fn increment_rejected(&mut self) {
        self.private.rejected += 1;
        debug!("{} sum2 messages rejected", self.private.rejected);
    }

    fn increment_discarded(&mut self) {
        self.private.discarded += 1;
        debug!("{} sum2 messages discarded", self.private.discarded);
    }
}

impl<C, M> PhaseState<Sum2, C, M>
where
    C: CoordinatorStorage,
    M: ModelStorage,
{
    /// Creates a new sum2 state.
    pub fn new(shared: Shared<C, M>, model_agg: Aggregation) -> Self {
        Self {
            private: Sum2 {
                model_agg,
                accepted: 0,
                rejected: 0,
                discarded: 0,
            },
            shared,
        }
    }

    /// Updates the mask dict with a sum2 participant request.
    async fn update_mask_dict(
        &mut self,
        participant_pk: SumParticipantPublicKey,
        model_mask: MaskObject,
    ) -> Result<(), RequestError> {
        self.shared
            .store
            .incr_mask_score(&participant_pk, &model_mask)
            .await?
            .into_inner()
            .map_err(RequestError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serial_test::serial;

    use super::*;
    use crate::{
        state_machine::{
            events::Event,
            tests::{
                builder::StateMachineBuilder,
                utils::{self, Participant},
            },
        },
        storage::tests::init_store,
    };
    use xaynet_core::{
        common::{RoundParameters, RoundSeed},
        crypto::{ByteObject, EncryptKeyPair},
        mask::{FromPrimitives, Model},
        SumDict,
    };

    impl Sum2 {
        pub fn aggregation(&self) -> &Aggregation {
            &self.model_agg
        }
    }

    #[tokio::test]
    #[serial]
    pub async fn integration_sum2_to_unmask() {
        utils::enable_logging();
        let model_length = 4;
        let round_params = RoundParameters {
            pk: EncryptKeyPair::generate().public,
            sum: 0.5,
            update: 1.0,
            seed: RoundSeed::generate(),
            mask_config: utils::mask_config(),
            model_length,
        };

        let n_updaters = 1;
        let n_summers = 1;

        // Generate a sum dictionary with a single sum participant
        let summer = utils::generate_summer(round_params.clone());
        let mut sum_dict = SumDict::new();
        sum_dict.insert(summer.keys.public, summer.ephm_keys.public);

        // Generate a new masked model, seed dictionary and aggregation
        let updater = utils::generate_updater(round_params.clone());
        let scalar = 1.0 / (n_updaters as f64 * round_params.update);
        let model = Model::from_primitives(vec![0; model_length].into_iter()).unwrap();
        let (mask_seed, masked_model) = updater.compute_masked_model(&model, scalar);
        let local_seed_dict = Participant::build_seed_dict(&sum_dict, &mask_seed);

        // Build the update seed dict that we'll give to the sum
        // participant, so that they can compute a global mask.
        let mut update_seed_dict = HashMap::new();
        let encrypted_seed = local_seed_dict.get(&summer.keys.public).unwrap();
        update_seed_dict.insert(updater.keys.public, encrypted_seed.clone());

        // Create the state machine in the Sum2 phase
        let mut agg = Aggregation::new(summer.mask_settings, model_length);
        agg.aggregate(masked_model);

        let mut store = init_store().await;
        let (state_machine, request_tx, events) = StateMachineBuilder::new(store.clone())
            .with_seed(round_params.seed.clone())
            .with_phase(Sum2 {
                model_agg: agg,
                accepted: 0,
                rejected: 0,
                discarded: 0,
            })
            .with_sum_ratio(round_params.sum)
            .with_update_ratio(round_params.update)
            .with_min_sum_count(n_summers)
            .with_max_sum_count(n_summers + 10)
            .with_min_update_count(n_updaters)
            .with_max_update_count(n_updaters + 10)
            .with_min_sum_time(1)
            .with_max_sum_time(2)
            .with_mask_config(utils::mask_settings().into())
            .build();
        assert!(state_machine.is_sum2());

        // Write the sum participant into the store so that the method store.incr_mask_score does
        // not fail
        store
            .add_sum_participant(&summer.keys.public, &summer.ephm_keys.public)
            .await
            .unwrap();

        // aggregate the masks (there's only one), compose a sum2
        // message and have the state machine process it
        let seeds = summer.decrypt_seeds(&update_seed_dict);
        let aggregation = summer.aggregate_masks(model_length, &seeds);
        let msg = summer.compose_sum2_message(aggregation.clone().into());

        let req = async { request_tx.msg(&msg).await.unwrap() };
        let transition = async { state_machine.next().await.unwrap() };
        let ((), state_machine) = tokio::join!(req, transition);
        assert!(state_machine.is_unmask());

        // Extract state of the state machine
        let PhaseState {
            private: unmask_state,
            ..
        } = state_machine.into_unmask_phase_state();

        // Check the initial state of the unmask phase.
        let mut best_masks = store.best_masks().await.unwrap().unwrap();
        assert_eq!(best_masks.len(), 1);
        let (mask, count) = best_masks.pop().unwrap();
        assert_eq!(count, 1);

        let unmasked_model = unmask_state.aggregation().unwrap().clone().unmask(mask);
        assert_eq!(unmasked_model, model);

        assert_eq!(
            events.phase_listener().get_latest(),
            Event {
                round_id: 0,
                event: PhaseName::Sum2,
            }
        );
    }
}
